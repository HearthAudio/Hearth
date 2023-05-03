// Main handler for worker role

use std::future::Future;
use std::pin::Pin;
use crate::config::Config;
pub use crate::worker::connector::initialize_api;

pub mod connector;
pub mod queue_processor;
pub mod analytics_reporter;
pub mod bot_handler;

pub async fn initialize_worker(config: Config) {
    println!("Worker INIT");
    // Init server
    initialize_api(&config);
}