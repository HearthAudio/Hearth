use crate::config::Config;
use crate::utils::generic_connector::{initialize_producer, send_message_generic};
use crate::utils::initialize_consume_generic;
use crate::worker::errors::report_error;
use crate::worker::queue_processor::{
    process_job, JobID, ProcessorIPC, ProcessorIPCData, ProcessorIncomingAction,
};
use crate::worker::{JOB_CHANNELS, WORKER_GUILD_IDS};
use anyhow::Result;
use hearth_interconnect::messages::{Message, PingPongResponse};
use hearth_interconnect::worker_communication::Job;
use log::{debug, error, info};
use rdkafka::producer::FutureProducer;
use songbird::Songbird;
use std::sync::{Arc, OnceLock};
use tokio::sync::broadcast::{Receiver, Sender};
use tokio::sync::{broadcast, Mutex};

pub static WORKER_PRODUCER: OnceLock<Mutex<Option<FutureProducer>>> = OnceLock::new();

pub async fn queue_internal_job(job: Job, config: &Config, songbird: Option<Arc<Songbird>>) {
    if &job.worker_id == config.config.worker_id.as_ref().unwrap() {
        info!("Starting new worker");

        let proc_config = config.clone();

        let (tx_processor, _rx_processor): (Sender<ProcessorIPCData>, Receiver<ProcessorIPCData>) =
            broadcast::channel(16);

        let tx_arc = Arc::new(tx_processor);

        {
            // Scoped to minimize lock time
            WORKER_GUILD_IDS
                .get()
                .unwrap()
                .lock()
                .await
                .push(job.guild_id.clone());
            JOB_CHANNELS
                .get()
                .unwrap()
                .insert(JobID::Specific(job.job_id.clone()), tx_arc.clone());
        }

        let job_tx = tx_arc.clone();

        tokio::spawn(async move {
            process_job(job, &proc_config, job_tx, report_error, songbird).await;
        });
    }
}

pub async fn initialize_api(
    config: &Config,
    ipc: &mut ProcessorIPC,
    songbird: Option<Arc<Songbird>>,
    group_id: &String,
) {
    let broker = config.kafka.kafka_uri.to_string();
    let _x = config.clone();

    initialize_worker_consume(broker, config, ipc, songbird, group_id).await;
}

async fn parse_message_callback(
    message: Message,
    config: Config,
    _sender: Arc<Sender<ProcessorIPCData>>,
    songbird: Option<Arc<Songbird>>,
) -> Result<()> {
    debug!("WORKER GOT MSG: {:?}", message);
    match message {
        Message::DirectWorkerCommunication(dwc) => {
            if &dwc.worker_id == config.config.worker_id.as_ref().unwrap() {
                let job_id = dwc.job_id.clone();

                let channel = JOB_CHANNELS
                    .get()
                    .unwrap()
                    .get(&JobID::Specific(job_id.clone()));

                match channel {
                    Some(channel) => {
                        let result = channel.send(ProcessorIPCData {
                            action_type: ProcessorIncomingAction::Actions(dwc.action_type.clone()),
                            songbird: None,
                            job_id: JobID::Specific(job_id.clone()),
                            dwc: Some(dwc.clone()),
                            error_report: None,
                        });
                        match result {
                            Ok(_) => {}
                            Err(_e) => {
                                error!("Failed to route DWC job message");
                            }
                        }
                    }
                    None => {
                        error!("Failed to route DWC job message - Could not find channel!");

                        // This is a bit sketchy in the future we should probably shift to a better state sync system
                        // That runs if this happens to resync client and server state across the distributed system
                        // The main issue with this is that it will only really work with `join` commands
                        // As all other commands relly on a pre-existing state notion that will be non-existent in this case

                        // If job does not exist create it
                        queue_internal_job(
                            Job {
                                job_id: job_id.clone(),
                                worker_id: dwc.worker_id.clone(),
                                request_id: dwc.request_id.clone().unwrap(),
                                guild_id: dwc.guild_id.clone(),
                            },
                            &config,
                            songbird,
                        )
                        .await;

                        // We could attempt to perform the action again but this would involve waiting for confirmation
                        // And would not work most of the time anyway because of a desynced state notion instead we will
                        // Just create the job and hope the user retry's and it works. This is well... pretty bad
                        // In the future as a slight upgrade to this we can send an erorr report and have the client auto-retry
                        // If it was a `join` command otherwise throw an error
                    }
                }
            }
        }
        Message::InternalPingPongRequest => {
            let mut px = WORKER_PRODUCER.get().unwrap().lock().await;
            let p = px.as_mut();

            send_message(
                &Message::InternalPongResponse(PingPongResponse {
                    worker_id: config.config.worker_id.clone().unwrap(),
                }),
                &config.kafka.kafka_topic,
                &mut *p.unwrap(),
            )
            .await;
        }
        Message::InternalWorkerQueueJob(job) => {
            queue_internal_job(job, &config, songbird).await;
        }
        _ => {}
    }
    Ok(())
}

pub async fn initialize_worker_consume(
    brokers: String,
    config: &Config,
    ipc: &mut ProcessorIPC,
    songbird: Option<Arc<Songbird>>,
    group_id: &String,
) {
    let producer: FutureProducer = initialize_producer(&brokers, config);
    let _ = WORKER_PRODUCER.set(Mutex::new(Some(producer)));
    initialize_consume_generic(
        &brokers,
        config,
        parse_message_callback,
        ipc,
        initialized_callback,
        songbird,
        group_id,
    )
    .await;
}

async fn initialized_callback(_: Config) {}

pub async fn send_message(message: &Message, topic: &str, producer: &mut FutureProducer) {
    send_message_generic(message, topic, producer).await;
}
