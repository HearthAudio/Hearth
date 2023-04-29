// Internal connector
use std::cell::{Cell, RefCell, RefMut};
use kafka;
use openssl;

use std::env;
use std::process;
use std::rc::Rc;
use kafka::consumer::{Builder, Consumer};

use self::kafka::client::{FetchOffset, KafkaClient, SecurityConfig};

use self::openssl::ssl::{SslConnector, SslFiletype, SslMethod, SslVerifyMode};

use serde::Deserialize;

use std::mem;


pub fn initialize_api() {
    env_logger::init();
    let broker = "kafka-3b1bc0b9-maxall4-aea3.aivencloud.com:23552".to_owned();
    consume(vec![broker],vec!["communication"]);
}

// All other job communication is passed directly to worker instead of running through scheduler
#[derive(Deserialize,Debug)]
enum WorkerMessageType {
    WorkerAnalytics,
    QueueJob,
    DirectWorkerCommunication // NOT DECODED
}

#[derive(Deserialize,Debug)]
struct Analytics {
    cpu_usage: u8,
    memory_usage: u8,
    jobs_running: u32,
    disk_usage: u8
}

#[derive(Deserialize,Debug)]
struct QueueJob {
    guild_id: String,
    voice_channel_id: String,
    volume: u8
}

#[derive(Deserialize,Debug)]
struct WorkerMessage {
    message_type: WorkerMessageType,
    analytics: Some(Analytics),
    queue_job: Some(QueueJob)
    // add the other fields if you need them
}


pub fn consume(brokers: Vec<String>,topics: Vec<&str>) {

    // ~ OpenSSL offers a variety of complex configurations. Here is an example:
    let mut builder = SslConnector::builder(SslMethod::tls()).unwrap();
    builder.set_cipher_list("DEFAULT").unwrap();
    builder.set_verify(SslVerifyMode::PEER);

    let cert_file = "service.cert";
    let cert_key = "service.key";
    let ca_cert = "ca.pem";

    println!("loading cert-file={}, key-file={}", cert_file, cert_key);

    builder
        .set_certificate_file(cert_file, SslFiletype::PEM)
        .unwrap();
    builder
        .set_private_key_file(cert_key, SslFiletype::PEM)
        .unwrap();
    builder.check_private_key().unwrap();

    builder.set_ca_file(ca_cert).unwrap();

    let connector = builder.build();

    // ~ instantiate KafkaClient with the previous OpenSSL setup
    let client = KafkaClient::new_secure(
        brokers.to_owned(),
        SecurityConfig::new(connector)
    );

    let mut consumer = Consumer::from_client(client)
        .with_topic(String::from("api_hook"))
        .create()
        .unwrap();

    loop {
        let mss = consumer.poll().unwrap();
        if mss.is_empty() {
            println!("No messages available right now.");
        }

        for ms in mss.iter() {
            for m in ms.messages() {
                let key: Thing = serde_json::from_slice(&m.key).unwrap();
                println!(
                    "{}:{:?}",
                    ms.topic(),
                    key
                );
            }
            let _ = consumer.consume_messageset(ms);
        }
        consumer.commit_consumed().unwrap();
    }
}

