use crate::config::*;
use crate::scheduler::*;
use tokio::main;

mod config;

mod scheduler;

#[tokio::main]
async fn main() {
    // Load config
    let config = init_config();
    // Initialize scheduler
    let scheduler_handle = tokio::spawn(async move {
        initialize_scheduler()
    });
    let out = scheduler_handle.await.unwrap().await;
    println!("GOT {:?}", out);
}

