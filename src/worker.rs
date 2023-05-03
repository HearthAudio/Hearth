// Main handler for worker role

use std::future::Future;
use std::pin::Pin;
use flume::{Receiver, Sender};
use crate::config::Config;
use crate::IPCWebsocketConnector;
pub use crate::worker::connector::initialize_api;
use crate::worker::webhook_handler::WebsocketInterconnect;

pub mod connector;
pub mod queue_processor;
pub mod analytics_reporter;
pub mod webhook_handler;

pub async fn initialize_worker(config: Config,ipc: IPCWebsocketConnector) {
    println!("Worker INIT");
    // Init server
    initialize_api(&config,ipc);
}