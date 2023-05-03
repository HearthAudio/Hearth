use std::future;
use std::future::Future;
use std::pin::Pin;
use flume::{Receiver, Sender};

use futures::{FutureExt, join};
use futures::future::{join_all, lazy};
use log::info;
use tokio::runtime::{Builder, Handle};
use tokio::task::{JoinHandle, JoinSet};

use crate::config::*;
use crate::deco::print_intro;
use crate::logger::setup_logger;
use crate::scheduler::*;
use crate::worker::*;
use crate::worker::webhook_handler::{initialize_websocket, VoiceConnectData, WebsocketInterconnect};

mod config;

mod scheduler;

mod worker;

mod logger;

mod utils;

mod deco;

// This is a bit of a hack to get around annoying type issues
async fn initialize_scheduler_internal(config: Config) {
    initialize_scheduler(config).await;
}

async fn initialize_worker_internal(config: Config,tx : Sender<WebsocketInterconnect>,rx : Receiver<WebsocketInterconnect>) {
    initialize_worker(config,tx,rx).await;
}

//TODO Add fern as logging library instead of env_logger
#[tokio::main]
async fn main() {
    let (tx,rx) : (Sender<WebsocketInterconnect>,Receiver<WebsocketInterconnect>) = flume::unbounded();
    let websocket_tx = tx.clone();
    let websocket_rx = rx.clone();
    print_intro();
    // Load config
    let worker_config = init_config();
    let scheduler_config = worker_config.clone();
    let websocket_config = worker_config.clone();
    // Setup logger
    setup_logger();
    // Depending on roles initialize worker and or scheduler on separate threads
    let mut futures = vec![];
    if worker_config.roles.worker {
        let worker = tokio::spawn(async move {
            return initialize_worker_internal(worker_config,tx,rx).await;
        });
        futures.push(worker);
    }
    if scheduler_config.roles.scheduler {
        let scheduler = tokio::spawn(async move {
            return initialize_scheduler_internal(scheduler_config).await;
        });
        futures.push(scheduler);
    }
    let websocket = tokio::task::spawn(async move {
        // initialize_bot_instance(&worker_config).await;
        initialize_websocket(&websocket_config,websocket_tx,websocket_rx).await;
    });
    futures.push(websocket);
    futures::future::join_all(futures).await;
}