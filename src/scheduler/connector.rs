use std::sync::Arc;
use hearth_interconnect::messages::{Message};
use rdkafka::producer::FutureProducer;
// Internal connector
use crate::utils::initialize_consume_generic;
use snafu::Whatever;
use songbird::Songbird;
use tokio::runtime::Handle;

use crate::scheduler::distributor::{distribute_job, WORKERS};
use crate::config::Config;
use crate::utils::generic_connector::{initialize_producer, PRODUCER, send_message_generic};
use crate::worker::queue_processor::ProcessorIPC;

pub async fn initialize_api(config: &Config,ipc: &mut ProcessorIPC) {
    let broker = config.config.kafka_uri.to_owned();

    let producer : FutureProducer = initialize_producer(&broker,config.config.kafka_group_id.as_ref().unwrap());
    *PRODUCER.lock().unwrap() = Some(producer);

    initialize_scheduler_consume(broker,  config,ipc).await;
}

fn parse_message_callback(parsed_message: Message, _: &PRODUCER, config: &Config, _: &mut ProcessorIPC,_: Option<Arc<Songbird>>) -> Result<(),Whatever> {
    match parsed_message {
        Message::ExternalQueueJob(j) => {
            // Handle event listener
            let mut px = PRODUCER.lock().unwrap();
            let p = px.as_mut();

            let rt = Handle::current();

            rt.block_on(distribute_job(j, &mut *p.unwrap(), config));
        }
        Message::InternalWorkerAnalytics(_a) => {
            //TODO
        },
        Message::InternalPongResponse(r) => {
            WORKERS.lock().unwrap().push(r.worker_id);
        }
        _ => {}
    }
    Ok(())
}



pub async fn initialize_scheduler_consume(brokers: String,config: &Config,ipc: &mut ProcessorIPC) {
    initialize_consume_generic(&brokers, config, parse_message_callback,  ipc,&PRODUCER,initialized_callback,None).await;
}

fn initialized_callback(config: &Config) {
    //TODO: Test

    let mut px = PRODUCER.lock().unwrap();
    let p = px.as_mut();

    let rt = Handle::current();

    rt.block_on(send_message(&Message::InternalPingPongRequest,config.config.kafka_topic.as_str(),&mut *p.unwrap()));
}

pub async fn send_message(message: &Message, topic: &str, producer: &mut FutureProducer) {
    send_message_generic(message,topic,producer).await;
}
