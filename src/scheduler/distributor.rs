use std::sync::Mutex;
use hearth_interconnect::messages::{Message, MessageType};
use hearth_interconnect::worker_communication::Job;
use kafka::producer::Producer;
use log::error;
use once_cell::sync::Lazy;
use serde::{Serialize,Deserialize};
use nanoid::nanoid;
use crate::config::Config;
use crate::scheduler::connector::{send_message};
// Handles distribution across worker nodes via round robin or maybe another method?


static ROUND_ROBIN_INDEX: Lazy<Mutex<usize>> = Lazy::new(|| Mutex::new(0));
pub static WORKERS: Lazy<Mutex<Vec<String>>> = Lazy::new(|| Mutex::new(vec![]));

// TODO: Implement Adaptive load balancing instead of round robin
pub fn distribute_job(message : Message,producer: &mut Producer,_config: &Config) {
    let mut index_guard = ROUND_ROBIN_INDEX.lock().unwrap();
    let workers_guard = WORKERS.lock().unwrap();
    let job_id = nanoid!();
    let queue_job_request = message.queue_job_request;
    match queue_job_request {
        Some(request) => {
            let internal_message = &Message {
                message_type: MessageType::InternalWorkerQueueJob,
                analytics: None,
                queue_job_request: None,
                queue_job_internal: Some(Job {
                    guild_id: request.guild_id,
                    voice_channel_id: request.voice_channel_id,
                    job_id: job_id,
                }),
                request_id: message.request_id,
                worker_id: Some(workers_guard[*index_guard].clone()),
                direct_worker_communication: None,
                external_queue_job_response: None,
                job_event: None,
                error_report: None
            };
            send_message(internal_message,"communication",producer);
            *index_guard += 1;
            if *index_guard == workers_guard.len() {
                *index_guard = 0;
            }
        },
        None => error!("Failed to distribute job!")
    }


}