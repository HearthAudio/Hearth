// Internal connector
use crate::utils::initialize_consume_generic;

use kafka::producer::{Producer};
use crate::scheduler::distributor::{distribute_job};
use crate::config::ReconfiguredConfig;
use crate::utils::generic_connector::{Message, MessageType, send_message_generic};

pub fn initialize_api(config: &ReconfiguredConfig) {
    let broker = "kafka-185690f4-maxall4-aea3.aivencloud.com:23552".to_owned();
    initialize_worker_consume(vec![broker],config);
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
            println!("{:?}",parsed_message);
            // We don't need to parse this as the scheduler
        }
    }
}


pub fn initialize_worker_consume(brokers: Vec<String>,config: &ReconfiguredConfig) {
    initialize_consume_generic(brokers,config,parse_message_callback,"WORKER");
}

pub fn send_message(message: &Message, topic: &str, mut producer: &mut Producer) {
    send_message_generic(message,topic,producer);
}
