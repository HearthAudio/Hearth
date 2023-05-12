use hearth_interconnect::messages::{Message, MessageType};
// Internal connector
use crate::utils::initialize_consume_generic;

use kafka::producer::{Producer};
use snafu::Whatever;

use crate::scheduler::distributor::{distribute_job, WORKERS};
use crate::config::Config;
use crate::utils::generic_connector::{initialize_client, initialize_producer, PRODUCER, send_message_generic};
use crate::worker::queue_processor::ProcessorIPC;

pub fn initialize_api(config: &Config,ipc: &mut ProcessorIPC) {
    let broker = config.config.kafka_uri.to_owned();
    let brokers = vec![broker];

    let producer : Producer = initialize_producer(initialize_client(&brokers));
    *PRODUCER.lock().unwrap() = Some(producer);

    initialize_scheduler_consume(brokers,config,ipc);
}

fn parse_message_callback(parsed_message: Message, _: &PRODUCER, config: &Config, _: &mut ProcessorIPC) -> Result<(),Whatever> {
    match parsed_message {
        Message::ExternalQueueJob(j) => {
            // Handle event listener
            let mut px = PRODUCER.lock().unwrap();
            let p = px.as_mut();

            distribute_job(parsed_message, &mut *p.unwrap(), config);
        }
        Message::InternalWorkerAnalytics(a) => {
            //TODO
        },
        Message::InternalPongResponse => {
            WORKERS.lock().unwrap().push(parsed_message.worker_id.unwrap());
        }
        _ => {}
    }
    Ok(())
}



pub fn initialize_scheduler_consume(brokers: Vec<String>,config: &Config,ipc: &mut ProcessorIPC) {
    initialize_consume_generic(brokers, config, parse_message_callback,  ipc,&PRODUCER,initialized_callback);
}

fn initialized_callback(config: &Config) {
    let mut px = PRODUCER.lock().unwrap();
    let p = px.as_mut();
    send_message(&Message {
        message_type: MessageType::InternalPingPongRequest,
        analytics: None,
        queue_job_request: None,
        queue_job_internal: None,
        request_id: "".to_string(),
        worker_id: None,
        direct_worker_communication: None,
        external_queue_job_response: None,
        job_event: None,
        error_report: None,
    },config.config.kafka_topic.as_str(),&mut *p.unwrap());
}

pub fn send_message(message: &Message, topic: &str, producer: &mut Producer) {
    send_message_generic(message,topic,producer);
}
