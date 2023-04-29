use std::sync::Mutex;
use kafka::producer::Producer;
use once_cell::sync::Lazy;
use serde::{Serialize,Deserialize};
use nanoid::nanoid;
use crate::scheduler::connector::{initialize_client, initialize_producer, Message, MessageType, send_message};
// Handles distribution across worker nodes via round robin or maybe another method?

#[derive(Deserialize,Debug,Serialize)]
pub struct Job {
    pub guild_id: String,
    pub voice_channel_id: String,
    pub job_id: String
}

static ROUND_ROBIN: Lazy<Mutex<u16>> = Lazy::new(|| Mutex::new(0));

// TODO: Implement Adaptive load balancing instead of round robin
pub fn distribute_job(message : Message,producer: Producer,request_id: String) {
    let mut guard = ROUND_ROBIN.lock().unwrap();
    let job_id = nanoid!();
    let queue_job_request = message.queue_job_request.unwrap();

    let message = &Message {
        message_type: MessageType::InternalWorkerQueueJob,
        analytics: None,
        queue_job_request: None,
        queue_job_internal: Some(Job {
            guild_id: queue_job_request.guild_id,
            voice_channel_id: queue_job_request.voice_channel_id,
            job_id: job_id
        }),
        request_id: request_id,
        worker_id: Some(*guard),
    };

    send_message(message,"communication",producer);

    *guard += 1;

}