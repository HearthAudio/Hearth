use std::thread;
// Internal connector
use crate::utils::initialize_consume_generic;

use kafka::producer::{Producer};
use log::{debug, info};
use tokio::task::spawn_blocking;
use crate::scheduler::distributor::{distribute_job};
use crate::config::ReconfiguredConfig;
use crate::utils::generic_connector::{Message, MessageType, send_message_generic};
use crate::worker::queue_processor::process_job;

pub fn initialize_api(config: &ReconfiguredConfig) {
    let broker = "kafka-185690f4-maxall4-aea3.aivencloud.com:23552".to_owned();
    initialize_worker_consume(vec![broker],config);
}

async fn test() {
    loop {
        println!("HELLO WORLD!")
    }
}

fn parse_message_callback(parsed_message: Message,mut producer: &mut Producer,config: &ReconfiguredConfig) {
    match parsed_message.message_type {
        MessageType::ExternalQueueJob => {},  // We don't need to parse this as the worker
        MessageType::InternalWorkerAnalytics => {}, // We don't need to parse this as the worker
        // Parseable
        MessageType::DirectWorkerCommunication => {
            // TODO
        },
        MessageType::InternalWorkerQueueJob => {
            debug!("{:?}",parsed_message);
            let handler = thread::spawn(|| {
                // thread code
                process_job(parsed_message)
            });
            // res.await;
        }
    }
}


pub fn initialize_worker_consume(brokers: Vec<String>,config: &ReconfiguredConfig) {
    initialize_consume_generic(brokers,config,parse_message_callback,"WORKER");
}

pub fn send_message(message: &Message, topic: &str, mut producer: &mut Producer) {
    send_message_generic(message,topic,producer);
}
