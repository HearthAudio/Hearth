use std::process;
use std::sync::Arc;

use log::{error, info, warn};
use sentry::ClientInitGuard;
use tokio::signal;
use crate::config::*;
use crate::deco::{print_intro, print_warnings};
use crate::logger::{setup_logger};
use crate::scheduler::*;
use crate::worker::*;


use tokio::sync::broadcast;
use tokio::sync::broadcast::{Receiver, Sender};

use crate::platform::check_platform_supported;
use crate::worker::queue_processor::{ProcessorIPC, ProcessorIPCData};

mod config;

mod scheduler;

mod worker;

mod logger;

mod utils;

mod deco;
mod platform;
mod extensions;

// This is a bit of a hack to get around annoying type issues
async fn initialize_scheduler_internal(config: Config,songbird_ipc: &mut ProcessorIPC) {
    initialize_scheduler(config,songbird_ipc).await;
}

async fn initialize_worker_internal(config: Config, songbird_ipc: &mut ProcessorIPC) {
    initialize_worker(config,songbird_ipc).await;
}


#[tokio::main]
async fn main() {
    print_intro();
    print_warnings();
    // Check if our platform is supported
    let platform_check = check_platform_supported();
    match platform_check {
        Ok(res) => {
            if res {
                warn!("Hearth may or may not work when running on MacOS with Apple Silicon.");
            }
        },
        Err(e) => error!("Failed to get system OS with error: {}", e)
    }
    // Load config
    let worker_config = init_config();
    let scheduler_config = worker_config.clone();
    let shutdown_config = worker_config.clone();

    // Setup logger
    setup_logger(&worker_config).expect("Logger Setup Failed - A bit ironic no?");

    // Setup Sentry
    let sentry : ClientInitGuard;
    if worker_config.config.sentry_url.is_some() {
        sentry = sentry::init((worker_config.config.sentry_url.clone().unwrap(), sentry::ClientOptions {
            release: sentry::release_name!(),
            ..Default::default()
        }));
        if sentry.is_enabled() {
            info!("Sentry Logger Enabled!");
        }
    }

    // Setup IPC
    let (tx_processor, _rx_processor) : (Sender<ProcessorIPCData>,Receiver<ProcessorIPCData>) = broadcast::channel(16);
    let scheduler_rx = tx_processor.subscribe();
    let worker_rx = tx_processor.subscribe();
    let tx_main = Arc::new(tx_processor);
    let mut worker_ipc = ProcessorIPC {
        sender: tx_main.clone(),
        receiver: worker_rx,
    };
    let mut scheduler_ipc = ProcessorIPC {
        sender: tx_main.clone(),
        receiver: scheduler_rx,
    };

    // Depending on roles initialize worker and or scheduler on separate threads
    if worker_config.roles.worker {
        let _worker = tokio::spawn(async move {
            initialize_worker_internal(worker_config, &mut worker_ipc).await;
        });
    }
    if scheduler_config.roles.scheduler {
        let _scheduler = tokio::spawn(async move {
            initialize_scheduler_internal(scheduler_config, &mut scheduler_ipc).await;
        });
    }

    match signal::ctrl_c().await {
        Ok(()) => {
            warn!("Initiating Graceful Shutdown! Do not attempt to cancel!");
            if shutdown_config.roles.worker {
                gracefully_shutdown_worker(&shutdown_config).await;
                info!("Graceful shutdown complete!");
            }
        },
        Err(err) => {
            error!("Unable to listen for shutdown signal: {}", err);
            // we also shut down in case of error
        },
    }
}