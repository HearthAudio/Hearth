use std::future;
use std::future::Future;
use std::pin::Pin;
use crate::config::*;
use crate::scheduler::*;
use crate::worker::*;
use tokio::main;

mod config;

mod scheduler;

mod worker;

mod logger;

use futures::future::{join_all, lazy};
use futures::{FutureExt, join};
use tokio::runtime::Builder;
use tokio::task::{JoinError, JoinHandle, JoinSet};
use crate::deco::print_intro;
use crate::logger::setup_logger;

mod utils;

mod deco;

// This is a bit of a hack to get around annoying type issues
async fn initialize_scheduler_internal(config: ReconfiguredConfig) {
    initialize_scheduler(config).await;
}

async fn initialize_worker_internal(config: ReconfiguredConfig) {
    initialize_worker(config).await;
}

//TODO Add fern as logging library instead of env_logger
#[tokio::main]
async fn main() {
    print_intro();
    // Load config
    // TODO: This is a bit of a hack we should probably pass config by pointer
    let worker_config = init_config();
    let scheduler_config = worker_config.clone();
    setup_logger();
    //
    let mut futures = vec![];
    if worker_config.roles.worker {
        let worker = tokio::spawn(async move {
            return initialize_worker_internal(worker_config).await;
        });
        futures.push(worker);
    }
    if scheduler_config.roles.scheduler {
        let scheduler = tokio::spawn(async move {
            return initialize_scheduler_internal(scheduler_config).await;
        });
        futures.push(scheduler);
    }
    futures::future::join_all(futures).await;
}