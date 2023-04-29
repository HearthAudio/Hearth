// Main handler for scheduler role

use crate::scheduler::connector::initialize_api;

mod connector;
mod distributor;


pub async fn initialize_scheduler() {
    println!("Scheduler INIT");
    // Init server
    initialize_api()
}