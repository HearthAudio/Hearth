// Main handler for scheduler role

use actix_web::dev::Server;
use crate::scheduler::API::initialize_api;

mod API;


pub async fn initialize_scheduler() {
    println!("Scheduler INIT");
    // Init server
    initialize_api()
}