// Internal connector





use std::sync::{Arc};

use hearth_interconnect::messages::Message;
use rdkafka::Message as KafkaMessage;
use log::{debug, error};

use rdkafka::{ClientConfig};
use rdkafka::consumer::{Consumer, StreamConsumer};
use rdkafka::producer::{FutureProducer, FutureRecord};
use songbird::Songbird;
use crate::config::Config;
use crate::utils::constants::KAFKA_SEND_TIMEOUT;
use crate::worker::queue_processor::{ProcessorIPC, ProcessorIPCData};
use async_fn_traits::{AsyncFn1, AsyncFn4};
use tokio::sync::broadcast::Sender;
use anyhow::{Result};


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
        .expect("Failed to create Producer");
    return producer;
}


pub async fn initialize_consume_generic(brokers: &String,  config: &Config, callback: impl AsyncFn4<Message, Config,Arc<Sender<ProcessorIPCData>>,Option<Arc<Songbird>>,Output = Result<()>>, ipc: &mut ProcessorIPC, initialized_callback: impl AsyncFn1<Config, Output = ()>,songbird: Option<Arc<Songbird>>,group_id: &String) {

    let mut consumer : StreamConsumer;
    if config.kafka.kafka_use_ssl {
        consumer = ClientConfig::new()
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
            .expect("Failed to create Kafka Configuration");
    } else {
        consumer = ClientConfig::new()
            .set("group.id", group_id)
            .set("bootstrap.servers", brokers)
            .set("enable.partition.eof", "false")
            .set("session.timeout.ms", "6000")
            .set("enable.auto.commit", "true")
            .clone()
            .create()
            .expect("Failed to create Kafka Configuration");
    }

    consumer
        .subscribe(&[&config.kafka.kafka_topic])
        .expect("Can't subscribe to specified topic");



    initialized_callback(config.clone()).await; // Unfortunate clone because of Async trait
    loop {
        let mss = consumer.recv().await;
        match mss {
            Ok(m) => {
                let payload = m.payload();
                
                match payload {
                    Some(payload) => {
                        let parsed_message : Result<Message,serde_json::Error> = serde_json::from_slice(payload);

                        match parsed_message {
                            Ok(m) => {
                                let _parse = callback(m,config.clone(),ipc.sender.clone(),songbird.clone()).await; // More Unfortunate clones because of Async trait. At least most of these implement Arc so it's not the worst thing in the world
                                // match parse {
                                //     Ok(_) => {},
                                //     Err(e) => error!("Failed to parse message with error: {}",e)
                                // }
                            },
                            Err(e) => error!("{}",e)
                        }
                    },
                    None => {
                        error!("Received No Payload!");
                    }
                    
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
