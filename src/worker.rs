// Main handler for worker role

pub mod actions;
pub mod analytics_reporter;
pub mod connector;
pub mod constants;
pub mod errors;
pub mod expiration;
pub mod helpers;
pub mod queue_processor;
pub mod serenity_handler;

use crate::config::Config;
use crate::worker::connector::{initialize_api, send_message, WORKER_PRODUCER};
use crate::worker::expiration::init_expiration_timer;
use crate::worker::queue_processor::JobID;
use crate::worker::queue_processor::{ProcessorIPC, ProcessorIPCData};
use crate::worker::serenity_handler::initialize_songbird;
use dashmap::DashMap;
use hearth_interconnect::messages::{Message, ShutdownAlert};
use log::info;
use nanoid::nanoid;
use std::sync::{Arc, OnceLock};
use tokio::sync::broadcast::Sender;
use tokio::sync::Mutex;
// .push(job.guild_id.clone());
static WORKER_GUILD_IDS: OnceLock<Mutex<Vec<String>>> = OnceLock::new();
pub static JOB_CHANNELS: OnceLock<DashMap<JobID, Arc<Sender<ProcessorIPCData>>>> = OnceLock::new();

pub async fn initialize_worker(config: Config, ipc: &mut ProcessorIPC) {
    info!("Worker INIT");

    JOB_CHANNELS.set(DashMap::new()).unwrap();
    let _ = WORKER_GUILD_IDS.set(Mutex::new(vec![]));
    //
    let songbird = initialize_songbird(&config, ipc).await;

    init_expiration_timer(ipc.sender.clone());
    initialize_api(&config, ipc, songbird, &nanoid!()).await;
}

pub async fn gracefully_shutdown_worker(config: &Config) {
    let worker_guild_ids = WORKER_GUILD_IDS.get().unwrap().lock().await;

    let mut px = WORKER_PRODUCER.get().unwrap().lock().await;
    let p = px.as_mut();

    send_message(
        &Message::WorkerShutdownAlert(ShutdownAlert {
            worker_id: config.config.worker_id.clone().unwrap(),
            affected_guild_ids: (*worker_guild_ids.clone()).to_owned(), // This isn't great but we have to do it to send the kafka message
        }),
        &config.kafka.kafka_topic,
        p.unwrap(),
    )
    .await;
}
