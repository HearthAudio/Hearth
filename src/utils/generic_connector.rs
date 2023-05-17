// Internal connector




use std::process::Output;
use std::sync::{Arc};
use tokio::sync::Mutex;
use hearth_interconnect::messages::Message;
use rdkafka::Message as KafkaMessage;
use log::{debug, error, info};

use rdkafka::{ClientConfig};
use rdkafka::consumer::{Consumer, StreamConsumer};
use rdkafka::producer::{FutureProducer, FutureRecord};
use songbird::Songbird;
use crate::config::Config;
use crate::utils::constants::KAFKA_SEND_TIMEOUT;
use crate::worker::queue_processor::{ProcessorIPC, ProcessorIPCData};
use async_fn_traits::{AsyncFn0, AsyncFn1, AsyncFn4, AsyncFn5, AsyncFnOnce1};
use tokio::sync::broadcast::Sender;
use anyhow::{Context, Result};


pub fn initialize_producer(brokers: &String) -> FutureProducer {
    let producer: FutureProducer = ClientConfig::new()
        .set("bootstrap.servers", brokers)
        .set("security.protocol","ssl")
        //SSL
        .set("ssl.ca.location","ca.pem")
        .set("ssl.certificate.location","service.cert")
        .set("ssl.key.location","service.key")
        .clone()
        .create()
        .unwrap();
    return producer;
}


pub async fn initialize_consume_generic(brokers: &String,  config: &Config, callback: impl AsyncFn4<Message, Config,Arc<Sender<ProcessorIPCData>>,Option<Arc<Songbird>>,Output = Result<()>>, ipc: &mut ProcessorIPC, initialized_callback: impl AsyncFn1<Config, Output = ()>,songbird: Option<Arc<Songbird>>,group_id: &String) {

    let consumer : StreamConsumer = ClientConfig::new()
        .set("group.id", group_id)
        .set("bootstrap.servers", brokers)
        .set("enable.partition.eof", "false")
        .set("session.timeout.ms", "6000")
        .set("enable.auto.commit", "true")
        .set("security.protocol","ssl")
        //SSL
        .set("ssl.ca.location","ca.pem")
        .set("ssl.certificate.location","service.cert")
        .set("ssl.key.location","service.key")
        .clone()
        .create()
        .unwrap();

    consumer
        .subscribe(&[&config.config.kafka_topic])
        .expect("Can't subscribe to specified topic");



    initialized_callback(config.clone()).await; // Unfortunate clone because of Async trait
    loop {
        let mss = consumer.recv().await;
        match mss {
            Ok(m) => {
                let payload = m.payload().unwrap();

                let parsed_message : Result<Message,serde_json::Error> = serde_json::from_slice(payload);

                match parsed_message {
                    Ok(m) => {
                        let parse = callback(m,config.clone(),ipc.sender.clone(),songbird.clone()).await; // More Unfortunate clones because of Async trait. At least most of these implement Arc so it's not the worst thing in the world
                        // match parse {
                        //     Ok(_) => {},
                        //     Err(e) => error!("Failed to parse message with error: {}",e)
                        // }
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
    debug!("Sent MSG");
}
