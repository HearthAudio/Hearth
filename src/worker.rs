// Main handler for worker role

use log::info;
use crate::config::Config;
pub use crate::worker::connector::initialize_api;
use crate::worker::songbird_handler::SongbirdIPC;

pub mod connector;
pub mod queue_processor;
pub mod analytics_reporter;
pub mod songbird_handler;

pub async fn initialize_worker(config: Config,songbird_ipc: &SongbirdIPC) {
    info!("Worker INIT");
    // Init server
    initialize_api(&config,songbird_ipc);
}