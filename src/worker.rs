// Main handler for worker role



use log::info;
use crate::config::Config;
use crate::worker::connector::initialize_api;
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
    let songbird = initialize_songbird(&config, ipc).await;
    init_expiration_timer(ipc.sender.clone());
    initialize_api(&config,ipc,songbird);
}