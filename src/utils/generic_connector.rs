// Internal connector

use kafka;
use openssl;


use std::process;
use kafka::consumer::{Consumer};

use self::kafka::client::{FetchOffset, KafkaClient, SecurityConfig};

use self::openssl::ssl::{SslConnector, SslFiletype, SslMethod, SslVerifyMode};

use serde::{Deserialize};


use std::time::Duration;
use kafka::producer::{Producer, Record, RequiredAcks};
use crate::scheduler::distributor::{Job};
use serde_derive::Serialize;
use crate::config::{ReconfiguredConfig};

// All other job communication is passed directly to worker instead of running through scheduler
#[derive(Deserialize,Debug,Serialize)]
#[serde(tag = "type")]
pub enum MessageType {
    // Internal
    InternalWorkerAnalytics,
    InternalWorkerQueueJob,
    // External
    ExternalQueueJob,
    // Other
    DirectWorkerCommunication,

}

#[derive(Deserialize,Debug,Serialize)]
pub struct Analytics {
    cpu_usage: u8,
    memory_usage: u8,
    jobs_running: u32,
    disk_usage: u8
}

#[derive(Deserialize,Debug,Serialize)]
pub struct JobRequest {
    pub guild_id: String,
    pub voice_channel_id: String
}

#[derive(Deserialize,Debug,Serialize)]
pub struct Message {
    pub message_type: MessageType, // Handles how message should be parsed
    pub analytics: Option<Analytics>, // Analytics sent by each worker
    pub queue_job_request: Option<JobRequest>,
    pub queue_job_internal: Option<Job>,
    pub request_id: String, // Unique string provided by client to identify this request
    pub worker_id: Option<usize> // ID Unique to each worker
}

pub fn initialize_client(brokers: &Vec<String>) -> KafkaClient {
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
    let mut client = KafkaClient::new_secure(
        brokers.to_owned(),
        SecurityConfig::new(connector)
    );

    // ~ communicate with the brokers
    match client.load_metadata_all() {
        Err(e) => {
            println!("{:?}", e);
            drop(client);
            process::exit(1);
        }
        Ok(_) => {
            // ~ at this point we have successfully loaded
            // metadata via a secured connection to one of the
            // specified brokers

            if client.topics().len() == 0 {
                println!("No topics available!");
            } else {
                // ~ now let's communicate with all the brokers in
                // the cluster our topics are spread over

                let topics: Vec<String> = client.topics().names().map(Into::into).collect();
                match client.fetch_offsets(topics.as_slice(), FetchOffset::Latest) {
                    Err(e) => {
                        println!("{:?}", e);
                        drop(client);
                        process::exit(1);
                    }
                    Ok(toffsets) => {
                        println!("Topic offsets:");
                        for (topic, mut offs) in toffsets {
                            offs.sort_by_key(|x| x.partition);
                            println!("{}", topic);
                            for off in offs {
                                println!("\t{}: {:?}", off.partition, off.offset);
                            }
                        }
                    }
                }
            }
        }
    }


    return client;

}

pub fn initialize_producer(client: KafkaClient) -> Producer {
    let mut producer = Producer::from_client(client)
        // ~ give the brokers one second time to ack the message
        .with_ack_timeout(Duration::from_secs(1))
        // ~ require only one broker to ack the message
        .with_required_acks(RequiredAcks::One)
        // ~ build the producer with the above settings
        .create().unwrap();
    return producer;
}


pub fn initialize_consume_generic(brokers: Vec<String>,config: &ReconfiguredConfig,callback: fn(Message, &mut Producer, &ReconfiguredConfig)) {

    let mut consumer = Consumer::from_client(initialize_client(&brokers))
        .with_topic(String::from("communication"))
        .create()
        .unwrap();

    let mut producer = initialize_producer(initialize_client(&brokers));

    loop {
        let mss = consumer.poll().unwrap();
        if mss.is_empty() {
            println!("No messages available right now.");
        }

        for ms in mss.iter() {
            for m in ms.messages() {
                let parsed_message : Result<Message,serde_json::Error> = serde_json::from_slice(&m.value);
                match parsed_message {
                    Ok(message) => {
                        callback(message,&mut producer, config);
                    },
                    Err(e) => println!("{} - Failed to parse message",e),
                }
            }
            let _ = consumer.consume_messageset(ms);
        }
        consumer.commit_consumed().unwrap();
    }
}

pub fn send_message_generic(message: &Message, topic: &str, mut producer: &mut Producer) {
    // Send message to worker
    let data = serde_json::to_string(message).unwrap();
    producer.send(&Record::from_value(topic, data)).unwrap();
}
