









use crate::config::*;
use crate::deco::print_intro;
use crate::logger::setup_logger;
use crate::scheduler::*;
use crate::worker::*;

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


#[tokio::main]
async fn main() {
    print_intro();
    // Load config
    let worker_config = init_config();
    let scheduler_config = worker_config.clone();
    // Setup logger
    setup_logger();
    // Depending on roles initialize worker and or scheduler on separate threads
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