// Main handler for worker role

use std::future::Future;
use std::pin::Pin;
use crate::config::Config;
use crate::worker::connector::initialize_api;

mod connector;
mod queue_processor;
mod analytics_reporter;
mod bot_handler;

pub async fn initialize_worker(config: Config) {
    println!("Worker INIT");
    // Init server
    initialize_api(&config);
}