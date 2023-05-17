use tokio::sync::Mutex;
use hearth_interconnect::messages::{JobRequest, Message};
use hearth_interconnect::worker_communication::Job;

use once_cell::sync::Lazy;

use nanoid::nanoid;
use rdkafka::producer::FutureProducer;
use crate::config::Config;
use crate::scheduler::connector::{send_message};
// Handles distribution across worker nodes via round robin or maybe another method?


static ROUND_ROBIN_INDEX: Lazy<Mutex<usize>> = Lazy::new(|| Mutex::new(0));
pub static WORKERS: Lazy<Mutex<Vec<String>>> = Lazy::new(|| Mutex::new(vec![]));

// TODO: Implement Adaptive load balancing instead of round robin
pub async fn distribute_job(job: JobRequest,producer: &mut FutureProducer,config: &Config) {
    let mut index_guard = ROUND_ROBIN_INDEX.lock().await;
    let workers_guard = WORKERS.lock().await;
    let job_id = nanoid!();
    let internal_message = &Message::InternalWorkerQueueJob(Job {
        job_id: job_id,
        worker_id: workers_guard[*index_guard].clone(),
        request_id: job.request_id
    });
    send_message(internal_message,config.config.kafka_topic.as_str(),producer).await;
    *index_guard += 1;
    if *index_guard == workers_guard.len() {
        *index_guard = 0;
    }


}