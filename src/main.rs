use std::sync::Arc;
use flume::{Receiver, Sender};
use crate::config::*;
use crate::deco::print_intro;
use crate::logger::setup_logger;
use crate::scheduler::*;
use crate::worker::*;
use crate::worker::songbird_handler::{initialize_songbird, SongbirdIPC, SongbirdRequestData};
use songbird::Songbird;

mod config;

mod scheduler;

mod worker;

mod logger;

mod utils;

mod deco;

// This is a bit of a hack to get around annoying type issues
async fn initialize_scheduler_internal(config: Config,songbird_ipc: &SongbirdIPC) {
    initialize_scheduler(config,songbird_ipc).await;
}

async fn initialize_worker_internal(config: Config,songbird_ipc: &SongbirdIPC) {
    initialize_worker(config,songbird_ipc).await;
}


#[tokio::main]
async fn main() {
    print_intro();
    // Setup logger
    setup_logger().expect("Logger Setup Failed - A bit ironic no?");
    // Load config
    let worker_config = init_config();
    let scheduler_config = worker_config.clone();
    let songbird_config = worker_config.clone();
    // Setup Flume Songbird IPC
    let (tx_songbird_request, rx_songbird_request) : (Sender<bool>,Receiver<bool>) = flume::bounded(2);
    let (tx_songbird_result, rx_songbird_result) : (Sender<Option<Arc<Songbird>>>,Receiver<Option<Arc<Songbird>>>) = flume::bounded(2);
    let songbird_ipc = SongbirdIPC {
        tx_songbird_request,
        rx_songbird_request,
        tx_songbird_result,
        rx_songbird_result
    };
    let songbird_ipc_worker = songbird_ipc.clone();
    let songbird_ipc_scheduler = songbird_ipc.clone();
    // Depending on roles initialize worker and or scheduler on separate threads
    let mut futures = vec![];
    if worker_config.roles.worker {
        let worker = tokio::spawn(async move {
            return initialize_worker_internal(worker_config,&songbird_ipc_worker).await;
        });
        let songbird = tokio::spawn(async move {
            return initialize_songbird(&songbird_config,&songbird_ipc).await;
        });
        futures.push(worker);
        futures.push(songbird);
    }
    if scheduler_config.roles.scheduler {
        let scheduler = tokio::spawn(async move {
            return initialize_scheduler_internal(scheduler_config,&songbird_ipc_scheduler).await;
        });
        futures.push(scheduler);
    }


    futures::future::join_all(futures).await;
}