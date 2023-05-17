use std::sync::Arc;

// use std::thread::sleep;


use hearth_interconnect::messages::{ExternalQueueJobResponse, Message, PingPongResponse};


use log::{debug, error, info};
use rdkafka::producer::FutureProducer;
use songbird::Songbird;






use crate::config::Config;
use crate::utils::generic_connector::{initialize_producer, send_message_generic};
// Internal connector
use crate::utils::initialize_consume_generic;

use crate::worker::errors::report_error;
use crate::worker::queue_processor::{JobID, process_job, ProcessorIncomingAction, ProcessorIPC, ProcessorIPCData};
use anyhow::{Result};
use lazy_static::lazy_static;
use tokio::sync::broadcast::Sender;
use tokio::sync::Mutex;

lazy_static! {
    pub static ref WORKER_PRODUCER: Mutex<Option<FutureProducer>> = Mutex::new(None);
}

pub async fn initialize_api(config: &Config, ipc: &mut ProcessorIPC,songbird: Option<Arc<Songbird>>,group_id: &String) {
    let broker = config.kafka.kafka_uri.to_string();
    let _x = config.clone();

    initialize_worker_consume(broker, config,ipc,songbird,group_id).await;
}

async fn parse_message_callback(message: Message, config: Config, sender: Arc<Sender<ProcessorIPCData>>,songbird: Option<Arc<Songbird>>) -> Result<()> {
    debug!("WORKER GOT MSG: {:?}",message);
    match message {
        Message::DirectWorkerCommunication(dwc) => {
            if &dwc.worker_id == config.config.worker_id.as_ref().unwrap() {
                let job_id = dwc.job_id.clone();
                let result = sender.send(ProcessorIPCData {
                    action_type: ProcessorIncomingAction::Actions(dwc.action_type.clone()),
                    songbird: None,
                    job_id: JobID::Specific(job_id.clone()),
                    dwc: Some(dwc),
                    error_report: None
                });
                match result {
                    Ok(_) => {},
                    Err(_e) => error!("Failed to send DWC to job: {}",&job_id),
                }
            }
        },
        Message::InternalPingPongRequest => {
            let mut px = WORKER_PRODUCER.lock().await;
            let p = px.as_mut();

            send_message(&Message::InternalPongResponse(PingPongResponse {
                worker_id: config.config.worker_id.clone().unwrap()
            }),&config.kafka.kafka_topic, &mut *p.unwrap()).await;
        }
        Message::InternalWorkerQueueJob(job) => {
            if &job.worker_id == config.config.worker_id.as_ref().unwrap() {
                let proc_config = config.clone();
                let job_id = job.job_id.clone();
                // This is a bit of a hack try and replace with tokio. Issue: Tokio task not executing when spawned inside another tokio task
                // rt.block_on(process_job(parsed_message, &proc_config, ipc.sender));
                // let sender = ipc.sender;
                let sender = sender.clone();
                info!("Starting new worker");

                tokio::spawn(async move {
                    process_job(job,&proc_config,sender,report_error,songbird).await;
                });

                let mut px = WORKER_PRODUCER.lock().await;
                let p = px.as_mut();

                send_message(&Message::ExternalQueueJobResponse(ExternalQueueJobResponse {
                    job_id,
                    worker_id: config.config.worker_id.as_ref().unwrap().clone(),
                }), config.kafka.kafka_topic.as_str(), &mut *p.unwrap()).await;
            }
        }
        _ => {}
    }
    Ok(())
}


pub async fn initialize_worker_consume(brokers: String,  config: &Config, ipc: &mut ProcessorIPC,songbird: Option<Arc<Songbird>>,group_id: &String) {
    let producer : FutureProducer = initialize_producer(&brokers);
    *WORKER_PRODUCER.lock().await = Some(producer);
    initialize_consume_generic(&brokers, config, parse_message_callback, ipc,initialized_callback,songbird,group_id).await;
}

async fn initialized_callback(_: Config) {
}

pub async fn send_message(message: &Message, topic: &str, producer: &mut FutureProducer) {
    send_message_generic(message,topic,producer).await;
}