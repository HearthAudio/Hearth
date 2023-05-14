use std::sync::Mutex;
use hearth_interconnect::messages::{JobRequest, Message};
use hearth_interconnect::worker_communication::Job;
use kafka::producer::Producer;

use once_cell::sync::Lazy;

use nanoid::nanoid;
use crate::config::Config;
use crate::scheduler::connector::{send_message};
// Handles distribution across worker nodes via round robin or maybe another method?


static ROUND_ROBIN_INDEX: Lazy<Mutex<usize>> = Lazy::new(|| Mutex::new(0));
pub static WORKERS: Lazy<Mutex<Vec<String>>> = Lazy::new(|| Mutex::new(vec![]));

// TODO: Implement Adaptive load balancing instead of round robin
pub fn distribute_job(job: JobRequest,producer: &mut Producer,config: &Config) {
    let mut index_guard = ROUND_ROBIN_INDEX.lock().unwrap();
    let workers_guard = WORKERS.lock().unwrap();
    let job_id = nanoid!();
    let internal_message = &Message::InternalWorkerQueueJob(Job {
        guild_id: job.guild_id,
        voice_channel_id: job.voice_channel_id,
        job_id: job_id,
        worker_id: workers_guard[*index_guard].clone(),
        request_id: job.request_id
    });
    send_message(internal_message,config.config.kafka_topic.as_str(),producer);
    *index_guard += 1;
    if *index_guard == workers_guard.len() {
        *index_guard = 0;
    }


}