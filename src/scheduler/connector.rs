// Internal connector
use crate::utils::initialize_consume_generic;

use kafka::producer::{Producer};
use crate::scheduler::distributor::{distribute_job};
use crate::config::Config;
use crate::utils::generic_connector::{initialize_client, initialize_producer, Message, MessageType, PRODUCER, send_message_generic};
use crate::worker::queue_processor::ProcessorIPC;

pub fn initialize_api(config: &Config,ipc: &mut ProcessorIPC) {
    let broker = config.config.kafka_uri.to_owned();
    initialize_scheduler_consume(vec![broker],config,ipc);
}

fn parse_message_callback(parsed_message: Message, mut producer: &PRODUCER, config: &Config, ipc: &mut ProcessorIPC) {
    match parsed_message.message_type {
        MessageType::ExternalQueueJob => {
            // Handle event listener
            let mut px = PRODUCER.lock().unwrap();
            let mut p = px.as_mut();

            distribute_job(parsed_message, &mut *p.unwrap(), config);
        }
        MessageType::InternalWorkerAnalytics => {
            //TODO
        },
        _ => {}
    }
}


pub fn initialize_scheduler_consume(brokers: Vec<String>,config: &Config,ipc: &mut ProcessorIPC) {
    let mut producer : Producer = initialize_producer(initialize_client(&brokers));
    *PRODUCER.lock().unwrap() = Some(producer);

    initialize_consume_generic(brokers, config, parse_message_callback, "WORKER", ipc,&PRODUCER);
}

pub fn send_message(message: &Message, topic: &str, producer: &mut Producer) {
    send_message_generic(message,topic,producer);
}
