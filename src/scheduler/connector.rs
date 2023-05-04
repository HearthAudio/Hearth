// Internal connector
use crate::utils::initialize_consume_generic;

use kafka::producer::{Producer};
use crate::scheduler::distributor::{distribute_job};
use crate::config::Config;
use crate::utils::generic_connector::{Message, MessageType, send_message_generic};
use crate::worker::songbird_handler::SongbirdIPC;

pub fn initialize_api(config: &Config,songbird_ipc: &SongbirdIPC) {
    let broker = "kafka-185690f4-maxall4-aea3.aivencloud.com:23552".to_owned();
    initialize_scheduler_consume(vec![broker],config,songbird_ipc);
}

fn parse_message_callback(parsed_message: Message,mut producer: &mut Producer,config: &Config,songbird_ipc: &SongbirdIPC) {
    match parsed_message.message_type {
        MessageType::ExternalQueueJob => {
            // Handle event listener
            distribute_job(parsed_message, &mut producer, config);
        }
        MessageType::InternalWorkerAnalytics => {
            //TODO
        },
        MessageType::DirectWorkerCommunication => {},   // We don't need to parse this as the scheduler
        MessageType::InternalWorkerQueueJob => {} // We don't need to parse this as the scheduler
    }
}


pub fn initialize_scheduler_consume(brokers: Vec<String>,config: &Config,songbird_ipc: &SongbirdIPC) {
    initialize_consume_generic(brokers,config,parse_message_callback,"SCHEDULER",songbird_ipc);
}

pub fn send_message(message: &Message, topic: &str, producer: &mut Producer) {
    send_message_generic(message,topic,producer);
}
