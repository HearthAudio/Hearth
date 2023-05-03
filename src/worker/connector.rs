use std::thread;

use kafka::producer::Producer;
use log::{info};
use tokio::runtime;

use crate::config::Config;

use crate::utils::generic_connector::{Message, MessageType, send_message_generic};
// Internal connector
use crate::utils::initialize_consume_generic;
use crate::worker::queue_processor::process_job;

pub fn initialize_api(config: &Config) {
    let broker = "kafka-185690f4-maxall4-aea3.aivencloud.com:23552".to_owned();
    initialize_worker_consume(vec![broker],config);
}

async fn test() {
    loop {
        println!("HELLO WORLD!")
    }
}

fn parse_message_callback(parsed_message: Message,_producer: &mut Producer,config: &Config) {
    //TODO: Check if this message is for us
    //TODO: Also worker ping pong stuff
    match parsed_message.message_type {
        MessageType::ExternalQueueJob => {},  // We don't need to parse this as the worker
        MessageType::InternalWorkerAnalytics => {}, // We don't need to parse this as the worker
        // Parseable
        MessageType::DirectWorkerCommunication => {
            // TODO
        },
        MessageType::InternalWorkerQueueJob => {
            let proc_config = config.clone();
            info!("{:?}",parsed_message);
            //TODO: This is a bit of a hack try and replace with tokio. Issue: Tokio task not executing when spawned inside another tokio task
            let rt = runtime::Handle::current();
            let _handler = thread::spawn(move || {
                // thread code
                rt.block_on(process_job(parsed_message, &proc_config));
            });
            // let scheduler = tokio::task::spawn(async move {
            //     // process_job(parsed_message, &proc_config).await;
            //     info!("XXX")
            // });
            // res.await;
        }
    }
}


pub fn initialize_worker_consume(brokers: Vec<String>,config: &Config) {
    initialize_consume_generic(brokers,config,parse_message_callback,"WORKER");
}

pub fn send_message(message: &Message, topic: &str, producer: &mut Producer) {
    send_message_generic(message,topic,producer);
}
