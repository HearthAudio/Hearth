use crate::config::*;
use crate::scheduler::*;
use tokio::main;

mod config;

mod scheduler;

mod utils;

#[tokio::main]
async fn main() {
    // Load config
    let config = init_config();
    // Initialize scheduler
    let scheduler_handle = tokio::spawn(async move {
        return initialize_scheduler(config)
    });
    let out = scheduler_handle.await.unwrap().await;
    println!("GOT {:?}", out);
}

