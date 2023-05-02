// Processing streaming jobs from kafka queue

// Parallelize with tokio

use log::info;
use crate::utils::generic_connector::Message;

pub fn process_job(message: Message) {
    // Expect and Unwrap are fine here because if we panic it will only panic the thread so it should be fine in most cases
    let queue_job = message.queue_job_internal.expect("Internal job empty!");
    info!("Job Processed!");
}