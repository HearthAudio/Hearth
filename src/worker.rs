// Main handler for worker role


pub mod connector;
pub mod queue_processor;
pub mod analytics_reporter;
pub mod serenity_handler;
pub mod actions;
pub mod errors;
pub mod helpers;
pub mod constants;
pub mod expiration;

use hearth_interconnect::messages::{Message, ShutdownAlert};
use lazy_static::lazy_static;
use log::info;
use nanoid::nanoid;
use crate::config::Config;
use dashmap::DashMap;
use tokio::sync::broadcast::Sender;
use crate::worker::connector::{initialize_api, send_message, WORKER_PRODUCER};
use crate::worker::expiration::init_expiration_timer;
use crate::worker::queue_processor::{ProcessorIPC, ProcessorIPCData};
use crate::worker::serenity_handler::initialize_songbird;
use crate::worker::queue_processor::JobID;
use tokio::sync::Mutex;
use std::sync::Arc;



pub async fn initialize_worker(config: Config, ipc: &mut ProcessorIPC) {
    info!("Worker INIT");
    //
    let songbird = initialize_songbird(&config, ipc).await;

    init_expiration_timer(ipc.sender.clone());
    initialize_api(&config,ipc,songbird,&nanoid!()).await;
}

pub async fn gracefully_shutdown_worker(config: &Config) {
    let worker_guild_ids = WORKER_GUILD_IDS.lock().await;

    let mut px = WORKER_PRODUCER.lock().await;
    let p = px.as_mut();

    send_message(&Message::WorkerShutdownAlert(ShutdownAlert {
        worker_id: config.config.worker_id.clone().unwrap(),
        affected_guild_ids: (*worker_guild_ids.clone()).to_owned(), // This isn't great but we have to do it to send the kafka message
    }), &config.kafka.kafka_topic, p.unwrap()).await;
}