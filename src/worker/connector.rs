use std::sync::Arc;
use std::thread;
use std::thread::JoinHandle;
use std::time::Instant;
use futures::executor;
use hashbrown::HashMap;

use kafka::producer::Producer;
use log::{info};
use openssl::version::dir;
use songbird::Songbird;
use tokio::runtime;
use tokio::sync::broadcast::Sender;

use crate::config::Config;

use crate::utils::generic_connector::{DirectWorkerCommunication, DWCActionType, ExternalQueueJobResponse, Message, MessageType, send_message_generic};
// Internal connector
use crate::utils::initialize_consume_generic;
use crate::worker::queue_processor::{LeaveAction, process_job, ProcessorIncomingAction, ProcessorIPC, ProcessorIPCData};
// use crate::worker::queue_processor::process_job;

pub fn initialize_api(config: &Config, ipc: &mut ProcessorIPC) {
    let broker = "kafka-185690f4-maxall4-aea3.aivencloud.com:23552".to_owned();
    initialize_worker_consume(vec![broker],config,ipc);
}

async fn test() {
    loop {
        println!("HELLO WORLD!")
    }
}

fn parse_message_callback(parsed_message: Message, producer: &mut Producer, config: &Config, mut ipc: &mut ProcessorIPC) {
    //TODO: Check if this message is for us
    //TODO: Also worker ping pong stuff
    match parsed_message.message_type {
        MessageType::ExternalQueueJob => {},  // We don't need to parse this as the worker
        MessageType::InternalWorkerAnalytics => {}, // We don't need to parse this as the worker
        MessageType::ExternalQueueJobResponse => {} // We don't need to parse this as the worker
        // Parseable
        MessageType::DirectWorkerCommunication => {
            // TODO
            match parsed_message.direct_worker_communication.unwrap().action_type {
                DWCActionType::LeaveChannel => {
                    let dwc = parsed_message.direct_worker_communication.unwrap();
                    ipc.sender.send(ProcessorIPCData {
                        action: ProcessorIncomingAction::LeaveChannel,
                        job_id: dwc.job_id,
                        songbird: None,
                        leave_action: Some(LeaveAction {
                            guild_id: dwc.leave_channel_guild_id.unwrap().parse().unwrap()
                        }),
                    }).expect("Sending DWC Failed");
                }
                _ => {}
            }
        },
        MessageType::InternalWorkerQueueJob => {
            let proc_config = config.clone();
            info!("{:?}",parsed_message);
            //TODO: This is a bit of a hack try and replace with tokio. Issue: Tokio task not executing when spawned inside another tokio task
            let rt = runtime::Handle::current();
            // rt.block_on(process_job(parsed_message, &proc_config, ipc.sender));
            // let sender = ipc.sender;
            let sender = ipc.sender.clone();
            let job_id = parsed_message.queue_job_internal.clone().unwrap().job_id;
            let request_id = parsed_message.request_id.clone();
            thread::spawn(move || {
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
    initialize_consume_generic(brokers,config,parse_message_callback,"WORKER",ipc);
}

pub fn send_message(message: &Message, topic: &str, producer: &mut Producer) {
    send_message_generic(message,topic,producer);
}
