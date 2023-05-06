

use std::thread;






use kafka::producer::Producer;
use log::{info};


use tokio::runtime;
use tokio::runtime::Builder;


use crate::config::Config;

use crate::utils::generic_connector::{ExternalQueueJobResponse, Message, MessageType, send_message_generic};
// Internal connector
use crate::utils::initialize_consume_generic;
use crate::worker::queue_processor::{process_job, ProcessorIncomingAction, ProcessorIPC, ProcessorIPCData};

pub fn initialize_api(config: &Config, ipc: &mut ProcessorIPC) {
    let broker = "kafka-185690f4-maxall4-aea3.aivencloud.com:23552".to_owned();
    initialize_worker_consume(vec![broker],config,ipc);
}

fn parse_message_callback(parsed_message: Message, producer: &mut Producer, config: &Config, ipc: &mut ProcessorIPC) {
    //TODO: Check if this message is for us
    //TODO: Also worker ping pong stuff
    match parsed_message.message_type {
        MessageType::ExternalQueueJob => {},  // We don't need to parse this as the worker
        MessageType::InternalWorkerAnalytics => {}, // We don't need to parse this as the worker
        MessageType::ExternalQueueJobResponse => {} // We don't need to parse this as the worker
        // Parseable
        MessageType::DirectWorkerCommunication => {
            let dwc = parsed_message.direct_worker_communication.unwrap();
            ipc.sender.send(ProcessorIPCData {
                action_type: ProcessorIncomingAction::Actions(dwc.action_type.clone()),
                songbird: None,
                dwc: Some(dwc)
            }).expect("Sending DWC Failed");
        },
        MessageType::InternalWorkerQueueJob => {
            let proc_config = config.clone();
            info!("{:?}",parsed_message);
            //TODO: This is a bit of a hack try and replace with tokio. Issue: Tokio task not executing when spawned inside another tokio task
            // rt.block_on(process_job(parsed_message, &proc_config, ipc.sender));
            // let sender = ipc.sender;
            let sender = ipc.sender.clone();
            let job_id = parsed_message.queue_job_internal.clone().unwrap().job_id;
            let request_id = parsed_message.request_id.clone();
            thread::spawn(move || {
                let rt = Builder::new_current_thread()
                    .enable_all()
                    .build()
                    .unwrap();
                rt.block_on(process_job(parsed_message, &proc_config, sender));
            });
            send_message(&Message {
                message_type: MessageType::ExternalQueueJobResponse,
                analytics: None,
                queue_job_request: None,
                queue_job_internal: None,
                request_id: request_id,
                worker_id: None,
                direct_worker_communication: None,
                external_queue_job_response: Some(ExternalQueueJobResponse {
                    job_id: Some(job_id)
                }),
                job_event: None,
            }, "communication", producer);
        }
    }
}


pub fn initialize_worker_consume(brokers: Vec<String>, config: &Config, ipc: &mut ProcessorIPC) {
    initialize_consume_generic(brokers, config, parse_message_callback, "WORKER", ipc);
}

pub fn send_message(message: &Message, topic: &str, producer: &mut Producer) {
    send_message_generic(message,topic,producer);
}
