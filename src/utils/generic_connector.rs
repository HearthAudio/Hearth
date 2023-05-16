// Internal connector


use std::future::Future;
use std::process;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use hearth_interconnect::messages::Message;
use rdkafka::Message as KafkaMessage;
use lazy_static::lazy_static;
use log::{debug, error, info, warn};
use openssl;
use rdkafka::{ClientConfig};
use rdkafka::consumer::{Consumer, StreamConsumer};
use rdkafka::producer::{FutureProducer, FutureRecord};


use snafu::Whatever;
use songbird::Songbird;
use crate::config::Config;
use crate::utils::constants::KAFKA_SEND_TIMEOUT;
use crate::worker::queue_processor::{ProcessorIPC};
use self::openssl::ssl::{SslConnector, SslFiletype, SslMethod, SslVerifyMode};

lazy_static! {
    pub static ref PRODUCER: Mutex<Option<FutureProducer>> = Mutex::new(None);
}

pub fn initialize_kafka_config(brokers: &String, group_id: &String) -> ClientConfig {
    ClientConfig::new()
        .set("group.id", group_id)
        .set("bootstrap.servers", brokers)
        .set("enable.partition.eof", "false")
        .set("session.timeout.ms", "6000")
        .set("enable.auto.commit", "false")
        .clone()
}

pub fn initialize_producer(brokers: &String, group_id: &String) -> FutureProducer {
    let producer: FutureProducer = initialize_kafka_config(brokers,group_id).create().unwrap();
    return producer;
}


pub async fn initialize_consume_generic(brokers: &String,  config: &Config, callback: fn(Message, &PRODUCER, &Config, &mut ProcessorIPC,Option<Arc<Songbird>>) -> Result<(),Whatever>, ipc: &mut ProcessorIPC, mut producer: &PRODUCER, initialized_callback: fn(&Config),songbird: Option<Arc<Songbird>>) {

    let consumer : StreamConsumer = initialize_kafka_config(brokers,&config.config.kafka_group_id.as_ref().unwrap()).create().unwrap();
    consumer
        .subscribe(&[&config.config.kafka_topic])
        .expect("Can't subscribe to specified topic");


    initialized_callback(&config);

    loop {
        let mss = consumer.recv().await;

        match mss {
            Ok(m) => {
                let payload = m.payload().unwrap();

                let parsed_message : Result<Message,serde_json::Error> = serde_json::from_slice(payload);

                match parsed_message {
                    Ok(m) => {
                        let parse = callback(m,&mut producer, config,ipc,songbird.clone());
                        match parse {
                            Ok(_) => {},
                            Err(e) => error!("Failed to parse message with error: {}",e)
                        }
                    },
                    Err(e) => error!("{}",e)
                }

            },
            Err(e) => error!("{}",e)
        }

    }
}

pub async fn send_message_generic(message: &Message, topic: &str, producer: &mut FutureProducer) {
    // Send message to worker
    let data = serde_json::to_string(message).unwrap();
    let record : FutureRecord<String,String> = FutureRecord::to(topic).payload(&data);
    producer.send(record, KAFKA_SEND_TIMEOUT).await.unwrap();
}
