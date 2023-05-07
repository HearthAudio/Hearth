// Internal connector
use crate::utils::initialize_consume_generic;

use kafka::producer::{Producer};
use crate::scheduler::distributor::{distribute_job};
use crate::config::Config;
use crate::utils::generic_connector::{Message, MessageType, send_message_generic};
use crate::worker::queue_processor::ProcessorIPC;

pub fn initialize_api(config: &Config,ipc: &mut ProcessorIPC) {
    let broker = config.config.kafka_uri.to_owned();
    initialize_scheduler_consume(vec![broker],config,ipc);
}

fn parse_message_callback(parsed_message: Message, producer: &mut Producer, config: &Config, _ipc: &mut ProcessorIPC) {
    match parsed_message.message_type {
        MessageType::ExternalQueueJob => {
            // Handle event listener
            distribute_job(parsed_message, producer, config);
        }
        MessageType::InternalWorkerAnalytics => {
            //TODO
        },
        _ => {}
    }
}


pub fn initialize_scheduler_consume(brokers: Vec<String>,config: &Config,ipc: &mut ProcessorIPC) {
    initialize_consume_generic(brokers,config,parse_message_callback,"SCHEDULER",ipc);
}

pub fn send_message(message: &Message, topic: &str, producer: &mut Producer) {
    send_message_generic(message,topic,producer);
}
