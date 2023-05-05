// Main handler for worker role

use std::thread::JoinHandle;
use hashbrown::HashMap;
use log::info;
use crate::config::Config;
use crate::worker::connector::initialize_api;
use crate::worker::queue_processor::ProcessorIPC;

pub mod connector;
pub mod queue_processor;
pub mod analytics_reporter;
pub mod songbird_handler;
pub mod direct_worker_communication;
pub mod sources;

pub async fn initialize_worker(config: Config, ipc: &mut ProcessorIPC) {
    info!("Worker INIT");
    // Init server
    initialize_api(&config,ipc);
}