// Main handler for scheduler role




use log::info;
use crate::config::Config;
use crate::scheduler::connector::initialize_api;
use crate::worker::queue_processor::ProcessorIPC;

mod connector;
pub(crate) mod distributor;


pub async fn initialize_scheduler(config: Config,ipc: &mut ProcessorIPC)  {
    info!("Scheduler INIT");
    // Init server
    initialize_api(&config,ipc).await;
}