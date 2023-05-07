use std::sync::Mutex;
use kafka::producer::Producer;
use log::error;
use once_cell::sync::Lazy;
use serde::{Serialize,Deserialize};
use nanoid::nanoid;
use crate::config::Config;
use crate::scheduler::connector::{send_message};
use crate::utils::generic_connector::{Message, MessageType};
// Handles distribution across worker nodes via round robin or maybe another method?

#[derive(Deserialize,Debug,Serialize,Clone)]
pub struct Job {
    pub guild_id: String,
    pub voice_channel_id: String,
    pub job_id: String
}

static ROUND_ROBIN: Lazy<Mutex<usize>> = Lazy::new(|| Mutex::new(0));

// TODO: Implement Adaptive load balancing instead of round robin
pub fn distribute_job(message : Message,producer: &mut Producer,_config: &Config) {
    let mut guard = ROUND_ROBIN.lock().unwrap();
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
                worker_id: Some(*guard),
                direct_worker_communication: None,
                external_queue_job_response: None,
                job_event: None,
                error_report: None
            };

            send_message(internal_message,"communication",producer);
            *guard += 1;
            //TODO: Create worker ID Ping Pong
            // if *guard == config.config.workers.len() {
            //     *guard = 0;
            // }
        },
        None => error!("Failed to distribute job!")
    }


}