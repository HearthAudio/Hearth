// Main handler for scheduler role

use std::future::Future;
use std::pin::Pin;
use crate::config::ReconfiguredConfig;
use crate::scheduler::connector::initialize_api;

mod connector;
pub(crate) mod distributor;


pub async fn initialize_scheduler(config: ReconfiguredConfig)  {
    println!("Scheduler INIT");
    // Init server
    initialize_api(&config);
}