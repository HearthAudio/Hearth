
use std::thread;
use hearth_interconnect::errors::ErrorReport;
use hearth_interconnect::messages::{Analytics, ExternalQueueJobResponse, JobRequest, Message, PingPongResponse};
use hearth_interconnect::worker_communication::{DirectWorkerCommunication, Job};
use serde_derive::Deserialize;
use serde_derive::Serialize;

use kafka::producer::Producer;

use log::{error, info};

use snafu::{OptionExt, Whatever};



use tokio::runtime::Builder;


use crate::config::Config;
use crate::utils::generic_connector::{ initialize_client, initialize_producer, PRODUCER, send_message_generic};
// Internal connector
use crate::utils::initialize_consume_generic;
use crate::worker::queue_processor::{process_job, ProcessorIncomingAction, ProcessorIPC, ProcessorIPCData};


pub fn initialize_api(config: &Config, ipc: &mut ProcessorIPC) {
    let broker = config.config.kafka_uri.to_owned();
    initialize_worker_consume(vec![broker],config,ipc);
}

// #[derive(Deserialize,Debug,Serialize,Clone)]
// #[serde(tag = "type")]
// pub enum Message {
//     InternalWorkerAnalytics(Analytics),
//     InternalWorkerQueueJob(Job),
//     InternalPingPongRequest,
//     InternalPongResponse,
//     // External
//     ExternalQueueJob(JobRequest),
//     ExternalQueueJobResponse(ExternalQueueJobResponse),
//     // Other
//     DirectWorkerCommunication(DirectWorkerCommunication),
//     ErrorReport(ErrorReport)
// }

pub fn report_error(error: ErrorReport,config: &Config) {
    error!("{}",error.error);

    // let mut px = PRODUCER.lock().unwrap();
    // let p = px.as_mut();
    // send_a(&Message {
    //     message_type: MessageType::ErrorReport,
    //     analytics: None,
    //     queue_job_request: None,
    //     queue_job_internal: None,
    //     request_id: error.request_id.clone(),
    //     worker_id: None,
    //     direct_worker_communication: None,
    //     external_queue_job_response: None,
    //     job_event: None,
    //     error_report: Some(error),
    // },config.config.kafka_topic.as_str(),&mut p.unwrap());
}

fn parse_message_callback(message: Message, _producer: &PRODUCER, config: &Config, ipc: &mut ProcessorIPC) -> Result<(),Whatever> {
    match message {
        Message::DirectWorkerCommunication(dwc) => {
            let job_id = dwc.job_id.clone();
            let result = ipc.sender.send(ProcessorIPCData {
                action_type: ProcessorIncomingAction::Actions(dwc.action_type.clone()),
                songbird: None,
                job_id: job_id.clone(),
                dwc: Some(dwc),
                error_report: None
            });
            match result {
                Ok(_) => {},
                Err(_e) => error!("Failed to send DWC to job: {}",&job_id),
            }
        },
        Message::InternalPingPongRequest => {
            let mut px = PRODUCER.lock().unwrap();
            let p = px.as_mut();
            send_message(&Message::InternalPongResponse(PingPongResponse {
                worker_id: config.config.worker_id.unwrap()
            }),config.config.kafka_topic.as_str(), &mut *p.unwrap());
        }
        Message::InternalWorkerQueueJob(job) => {
            let proc_config = config.clone();
            // This is a bit of a hack try and replace with tokio. Issue: Tokio task not executing when spawned inside another tokio task
            // rt.block_on(process_job(parsed_message, &proc_config, ipc.sender));
            // let sender = ipc.sender;
            let sender = ipc.sender.clone();
            info!("Starting new worker");
            thread::spawn(move || {
                let rt = Builder::new_current_thread()
                    .enable_all()
                    .build()
                    .unwrap();
                rt.block_on(process_job(job, &proc_config, sender,report_error));
            });
            let mut px = PRODUCER.lock().unwrap();
            let p = px.as_mut();

            send_message(&Message::ExternalQueueJobResponse(ExternalQueueJobResponse {
                job_id: job.job_id.clone(),
                worker_id: config.config.worker_id.as_ref().unwrap().clone(),
            }),config.config.kafka_topic.as_str(), &mut *p.unwrap());
        }
        _ => {}
    }
    Ok(())
}


pub fn initialize_worker_consume(brokers: Vec<String>, config: &Config, ipc: &mut ProcessorIPC) {
    let producer : Producer = initialize_producer(initialize_client(&brokers));
    *PRODUCER.lock().unwrap() = Some(producer);

    initialize_consume_generic(brokers, config, parse_message_callback, ipc,&PRODUCER,initialized_callback);
}

fn initialized_callback(_: &Config) {}

pub fn send_message(message: &Message, topic: &str, producer: &mut Producer) {
    send_message_generic(message,topic,producer);
}