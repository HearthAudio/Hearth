// Main handler for worker role

use std::future::Future;
use std::pin::Pin;
use crate::config::ReconfiguredConfig;
use crate::worker::connector::initialize_api;

mod connector;

pub async fn initialize_worker(config: ReconfiguredConfig) {
    println!("Worker INIT");
    // Init server
    initialize_api(&config);
}