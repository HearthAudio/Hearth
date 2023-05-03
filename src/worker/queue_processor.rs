// Processing streaming jobs from kafka queue

// Parallelize with tokio

use log::info;
use crate::config::Config;
use crate::utils::generic_connector::Message;

pub async fn process_job(message: Message, _config: &Config) {
    // Expect and Unwrap are fine here because if we panic it will only panic the thread so it should be fine in most cases
    let _queue_job = message.queue_job_internal.expect("Internal job empty!");
    info!("Job Processed!");
}