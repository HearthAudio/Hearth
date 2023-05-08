

use crate::config::*;
use crate::deco::{print_intro, print_warnings};
use crate::logger::setup_logger;
use crate::scheduler::*;
use crate::worker::*;
use crate::worker::serenity_handler::{initialize_songbird};

use tokio::sync::broadcast;
use tokio::sync::broadcast::{Receiver, Sender};
use crate::worker::queue_processor::{ProcessorIPC, ProcessorIPCData};

mod config;

mod scheduler;

mod worker;

mod logger;

mod utils;

mod deco;

// This is a bit of a hack to get around annoying type issues
async fn initialize_scheduler_internal(config: Config,songbird_ipc: &mut ProcessorIPC) {
    initialize_scheduler(config,songbird_ipc).await;
}

async fn initialize_worker_internal(config: Config, songbird_ipc: &mut ProcessorIPC) {
    initialize_worker(config,songbird_ipc).await;
}


#[tokio::main]
async fn main() {
    // let source = uri_stream("").await;
    print_intro();
    // Setup logger
    setup_logger().expect("Logger Setup Failed - A bit ironic no?");
    print_warnings();
    // Load config
    let worker_config = init_config();
    let scheduler_config = worker_config.clone();
    let songbird_config = worker_config.clone();
    // Setup Flume Songbird IPC
    let (tx_processor, _rx_processor) : (Sender<ProcessorIPCData>,Receiver<ProcessorIPCData>) = broadcast::channel(16);
    let songbird_rx = tx_processor.subscribe();
    let scheduler_rx = tx_processor.subscribe();
    let worker_rx = tx_processor.subscribe();
    let songbird_tx = tx_processor.clone();
    let scheduler_tx  = tx_processor.clone();
    let mut worker_ipc = ProcessorIPC {
        sender: tx_processor,
        receiver: worker_rx,
    };
    let mut songbird_ipc = ProcessorIPC {
        sender: songbird_tx,
        receiver: songbird_rx,
    };
    let mut scheduler_ipc = ProcessorIPC {
        sender: scheduler_tx,
        receiver: scheduler_rx,
    };
    // Depending on roles initialize worker and or scheduler on separate threads
    let mut futures = vec![];
    if worker_config.roles.worker {
        let worker = tokio::spawn(async move {
            return initialize_worker_internal(worker_config, &mut worker_ipc).await;
        });
        let songbird = tokio::spawn(async move {
            return initialize_songbird(&songbird_config, &mut songbird_ipc).await;
        });
        futures.push(worker);
        futures.push(songbird);
    }
    if scheduler_config.roles.scheduler {
        let scheduler = tokio::spawn(async move {
            return initialize_scheduler_internal(scheduler_config, &mut scheduler_ipc).await;
        });
        futures.push(scheduler);
    }


    futures::future::join_all(futures).await;
}