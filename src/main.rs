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
use futures::future::{join_all, lazy};
use futures::{FutureExt, join};
use tokio::runtime::Builder;
use tokio::task::{JoinError, JoinHandle, JoinSet};

mod utils;

// This is a bit of a hack to get around annoying type issues
async fn initialize_scheduler_internal(config: ReconfiguredConfig) {
    initialize_scheduler(config).await;
}

async fn initialize_worker_internal(config: ReconfiguredConfig) {
    initialize_worker(config).await;
}

#[tokio::main]
async fn main() {
    //TODO: Abide by config roles for scheduler and worker
    // Load config
    // TODO: This is a bit of a hack we should probably pass config by pointer
    let worker_config = init_config();
    let scheduler_config = worker_config.clone();
    env_logger::init();
    //
    let scheduler = tokio::spawn(async move {
        return initialize_scheduler_internal(scheduler_config).await;
    });
    let worker = tokio::spawn(async move {
       return initialize_worker_internal(worker_config).await;
    });
    let futures = vec![
        scheduler,
        worker,
    ];
    futures::future::join_all(futures).await;
}

//
// use std::time::Duration;
//
// fn work1() {
//     // std::thread::sleep(Duration::from_secs(5));
//     loop {
//         println!("HI1")
//     }
// }
//
// fn work2() {
//     loop {
//         println!("HI2")
//     }
// }
//
// #[tokio::main]
// async fn main() -> () {
//     let tasks = vec![
//         tokio::spawn(async move { work1() }),
//         tokio::spawn(async move { work2() }),
//     ];
//
//     futures::future::join_all(tasks).await;
// }
//
