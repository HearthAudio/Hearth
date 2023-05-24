use std::sync::Arc;
use hearth_interconnect::messages::{Message, PingPongResponse};
use log::{debug, error, info};
use rdkafka::producer::FutureProducer;
use songbird::Songbird;
use crate::config::Config;
use crate::utils::generic_connector::{initialize_producer, send_message_generic};
use crate::utils::initialize_consume_generic;
use crate::worker::errors::report_error;
use crate::worker::queue_processor::{JobID, process_job, ProcessorIncomingAction, ProcessorIPC, ProcessorIPCData};
use anyhow::{Result};
use hearth_interconnect::errors::ErrorReport;
use lazy_static::lazy_static;
use tokio::sync::broadcast::{Receiver, Sender};
use tokio::sync::{broadcast, Mutex};
use crate::worker::{JOB_CHANNELS, WORKER_GUILD_IDS};

lazy_static! {
    pub static ref WORKER_PRODUCER: Mutex<Option<FutureProducer>> = Mutex::new(None);
}

pub async fn initialize_api(config: &Config, ipc: &mut ProcessorIPC,songbird: Option<Arc<Songbird>>,group_id: &String) {
    let broker = config.kafka.kafka_uri.to_string();
    let _x = config.clone();

    initialize_worker_consume(broker, config,ipc,songbird,group_id).await;
}

async fn parse_message_callback(message: Message, config: Config, sender: Arc<Sender<ProcessorIPCData>>,songbird: Option<Arc<Songbird>>) -> Result<()> {
    debug!("WORKER GOT MSG: {:?}",message);
    match message {
        Message::DirectWorkerCommunication(dwc) => {
            if &dwc.worker_id == config.config.worker_id.as_ref().unwrap() {
                let job_id = dwc.job_id.clone();

                let channel = JOB_CHANNELS.get(&JobID::Specific(job_id.clone()));

                match channel {
                    Some(channel) => {
                        let result = channel.send(ProcessorIPCData {
                            action_type: ProcessorIncomingAction::Actions(dwc.action_type.clone()),
                            songbird: None,
                            job_id: JobID::Specific(job_id.clone()),
                            dwc: Some(dwc),
                            error_report: None
                        });
                        match result {
                            Ok(_) => {},
                            Err(_e) => error!("Failed to send DWC to job: {}",&job_id),
                        }
                    },
                    None => {
                        //TODO: Remove unwrap(s)
                        report_error(ErrorReport {
                            error: "Job does not exist".to_string(),
                            request_id: dwc.request_id.unwrap(),
                            job_id,
                            guild_id: dwc.guild_id,
                        },&config);
                    }
                }

            }
        },
        Message::InternalPingPongRequest => {
            let mut px = WORKER_PRODUCER.lock().await;
            let p = px.as_mut();

            send_message(&Message::InternalPongResponse(PingPongResponse {
                worker_id: config.config.worker_id.clone().unwrap()
            }),&config.kafka.kafka_topic, &mut *p.unwrap()).await;
        }
        Message::InternalWorkerQueueJob(job) => {
            if &job.worker_id == config.config.worker_id.as_ref().unwrap() {
                info!("Starting new worker");

                let proc_config = config.clone();

                let (tx_processor, _rx_processor) : (Sender<ProcessorIPCData>,Receiver<ProcessorIPCData>) = broadcast::channel(16);

                let tx_arc = Arc::new(tx_processor);

                { // Scoped to minimize lock time
                    WORKER_GUILD_IDS.lock().await.push(job.guild_id.clone());
                    JOB_CHANNELS.insert(JobID::Specific(job.job_id.clone()),tx_arc.clone());
                }

                let job_tx = tx_arc.clone();

                tokio::spawn(async move {
                    process_job(job,&proc_config,job_tx,report_error,songbird).await;
                });
            }
        }
        _ => {}
    }
    Ok(())
}


pub async fn initialize_worker_consume(brokers: String,  config: &Config, ipc: &mut ProcessorIPC,songbird: Option<Arc<Songbird>>,group_id: &String) {
    let producer : FutureProducer = initialize_producer(&brokers,config);
    *WORKER_PRODUCER.lock().await = Some(producer);
    initialize_consume_generic(&brokers, config, parse_message_callback, ipc,initialized_callback,songbird,group_id).await;
}

async fn initialized_callback(_: Config) {
}

pub async fn send_message(message: &Message, topic: &str, producer: &mut FutureProducer) {
    send_message_generic(message,topic,producer).await;
}