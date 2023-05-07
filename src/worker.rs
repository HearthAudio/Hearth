// Main handler for worker role



use log::info;
use crate::config::Config;
use crate::worker::connector::initialize_api;
use crate::worker::queue_processor::ProcessorIPC;

pub mod connector;
pub mod queue_processor;
pub mod analytics_reporter;
pub mod songbird_handler;
pub mod sources;
pub mod actions;

pub async fn initialize_worker(config: Config, ipc: &mut ProcessorIPC) {
    info!("Worker INIT");
    // Init server
    initialize_api(&config,ipc);
}