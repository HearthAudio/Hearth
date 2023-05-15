// Main handler for worker role



use std::time::Duration;
use log::info;
use tokio::time::sleep;
use crate::config::Config;
use crate::worker::actions::channel_manager::join_channel;
use crate::worker::connector::initialize_api;
use crate::worker::errors::report_error;
use crate::worker::expiration::init_expiration_timer;
use crate::worker::queue_processor::ProcessorIPC;
use crate::worker::serenity_handler::initialize_songbird;

pub mod connector;
pub mod queue_processor;
pub mod analytics_reporter;
pub mod serenity_handler;
pub mod actions;
pub mod errors;
pub mod helpers;
pub mod constants;
pub mod expiration;

pub async fn initialize_worker(config: Config, ipc: &mut ProcessorIPC) {
    info!("Worker INIT");
    //
    let mut songbird = initialize_songbird(&config, ipc).await;

    init_expiration_timer(ipc.sender.clone());
    initialize_api(&config,ipc,songbird).await;
}