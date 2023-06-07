use std::sync::{Arc, OnceLock};
use std::time::Duration;
use hearth_interconnect::messages::{Message};
use rdkafka::producer::FutureProducer;
// Internal connector
use crate::utils::initialize_consume_generic;
use songbird::Songbird;


use crate::scheduler::distributor::{distribute_job, ROUND_ROBIN_INDEX, WORKERS};
use crate::config::Config;
use crate::utils::generic_connector::{initialize_producer, send_message_generic};
use crate::worker::queue_processor::{ProcessorIPC, ProcessorIPCData};
use anyhow::{Result};
use hearth_interconnect::errors::ErrorReport;

use log::{debug, info, warn};
use tokio::sync::broadcast::Sender;

use tokio::sync::Mutex;
use crate::worker::errors::report_error;

pub static SCHEDULER_PRODUCER: OnceLock<Mutex<Option<FutureProducer>>> = OnceLock::new();

pub async fn initialize_api(config: &Config,ipc: &mut ProcessorIPC,group_id: &String) {
    let broker = config.kafka.kafka_uri.to_owned();

    let producer : FutureProducer = initialize_producer(&broker,config);

    SCHEDULER_PRODUCER.set(Mutex::new(Some(producer)));
    ROUND_ROBIN_INDEX.set(Mutex::new(0));
    WORKERS.set(Mutex::new(vec![]));

    initialize_scheduler_consume(broker, config,ipc,group_id).await;
}

async fn parse_message_callback(parsed_message: Message, config: Config, _: Arc<Sender<ProcessorIPCData>>,_: Option<Arc<Songbird>>) -> Result<()> {
    debug!("SCHEDULER GOT MSG: {:?}",parsed_message);
    match parsed_message {
        Message::ExternalQueueJob(j) => {
            // Handle event listener
            let mut px = SCHEDULER_PRODUCER.get().unwrap().lock().await;
            let p = px.as_mut();

            let rid = j.request_id.clone();
            let guild_id = j.guild_id.clone();

            let distribute = distribute_job(j, &mut *p.unwrap(), &config).await;
            match distribute {
                Ok(_) => {},
                Err(e) => {
                    report_error(ErrorReport {
                        error: e.to_string(),
                        request_id: rid,
                        job_id: "N/A".to_string(),
                        guild_id
                    },&config)
                }
            }
        },
        Message::WorkerShutdownAlert(shutdown_alert) => {
            let mut workers = WORKERS.get().unwrap().lock().await;
            workers.retain(|x| x != &shutdown_alert.worker_id); // Remove worker from possible workers in scheduler if shut down
            // We will also reset the round robin index here to make sure that if it is on the new worker it does not get stuck
            let mut index_guard = ROUND_ROBIN_INDEX.get().unwrap().lock().await;
            *index_guard = 0;
        },
        Message::InternalWorkerAnalytics(_a) => {
            //TODO
        },
        Message::InternalPongResponse(r) => {
            let mut workers = WORKERS.get().unwrap().lock().await;
            if !workers.contains(&r.worker_id) {
                workers.push(r.worker_id.clone());
                info!("ADDED NEW WORKER: {}",r.worker_id);
            }
        }
        _ => {}
    }
    Ok(())
}



pub async fn initialize_scheduler_consume(brokers: String,config: &Config,ipc: &mut ProcessorIPC,group_id: &String) {
    initialize_consume_generic(&brokers, config, parse_message_callback,  ipc,initialized_callback,None,group_id).await;
}

async fn initialized_callback(config: Config) {
    tokio::task::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_millis(3000)); //TODO: Exponential decrease. MAX: 10S from 1.5S over 60S TF
        let mut icounts = 0;
        loop {
            interval.tick().await;
            let mut px = SCHEDULER_PRODUCER.get().unwrap().lock().await;
            let p = px.as_mut();
            send_message(&Message::InternalPingPongRequest,config.kafka.kafka_topic.as_str(),&mut *p.unwrap()).await;
            icounts += 1;
            if icounts > 4 {
                let wg = WORKERS.get().unwrap().lock().await;
                if wg.len() == 0 {
                    warn!("Ping checking has been stopped. But no workers have been found! Make sure all of your workers are running!");
                    break;
                }
                info!("Ping Checking Stopped. Found: {} workers!",wg.len());
                break;
            }
        }
    });
}

pub async fn send_message(message: &Message, topic: &str, producer: &mut FutureProducer) {
    send_message_generic(message,topic,producer).await;
}
