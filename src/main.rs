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

async fn initialize_worker_internal(config: Config) {
    initialize_worker(config).await;
}

//TODO Add fern as logging library instead of env_logger
#[tokio::main]
async fn main() {
    print_intro();
    // Load config
    let worker_config = init_config();
    let scheduler_config = worker_config.clone();
    // Setup logger
    setup_logger();
    // Depending on roles initialize worker and or scheduler on separate threads
    // let mut futures = vec![];
    // if worker_config.roles.worker {
    //     let worker = tokio::spawn(async move {
    //         return initialize_worker_internal(worker_config).await;
    //     });
    //     futures.push(worker);
    // }
    // if scheduler_config.roles.scheduler {
    //     let scheduler = tokio::spawn(async move {
    //         let test = tokio::task::spawn(async move {
    //             // process_job(parsed_message, &proc_config).await;
    //             info!("XXX")
    //         });
    //         return initialize_scheduler_internal(scheduler_config).await;
    //     });
    //     futures.push(scheduler);
    // }
    // futures::future::join_all(futures).await;
    let (tx,rx) : (Sender<WebsocketInterconnect>,Receiver<WebsocketInterconnect>) = flume::unbounded();
    let test = tokio::task::spawn(async move {
        // initialize_bot_instance(&worker_config).await;
        initialize_websocket(&worker_config,tx,rx).await;
    });
    test.await.unwrap();
}