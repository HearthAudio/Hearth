// Main handler for scheduler role




use log::info;
use crate::config::Config;
use crate::scheduler::connector::initialize_api;
use crate::worker::songbird_handler::SongbirdIPC;

mod connector;
pub(crate) mod distributor;


pub async fn initialize_scheduler(config: Config,songbird_ipc: &SongbirdIPC)  {
    info!("Scheduler INIT");
    // Init server
    initialize_api(&config,songbird_ipc);
}