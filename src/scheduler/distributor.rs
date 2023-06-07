use crate::config::Config;
use crate::scheduler::connector::send_message;
use anyhow::{bail, Result};
use hearth_interconnect::messages::{JobRequest, Message};
use hearth_interconnect::worker_communication::Job;
use nanoid::nanoid;
use rdkafka::producer::FutureProducer;
use std::sync::OnceLock;
use tokio::sync::Mutex;
// Handles distribution across worker nodes via round robin or maybe another method?

pub static ROUND_ROBIN_INDEX: OnceLock<Mutex<usize>> = OnceLock::new();
pub static WORKERS: OnceLock<Mutex<Vec<String>>> = OnceLock::new();

pub async fn distribute_job(
    job: JobRequest,
    producer: &mut FutureProducer,
    config: &Config,
) -> Result<()> {
    let mut index_guard = ROUND_ROBIN_INDEX.get().unwrap().lock().await;
    let workers_guard = WORKERS.get().unwrap().lock().await;

    if workers_guard.len() == 0 {
        bail!("No Workers Registered! Can't distribute Job!")
    }

    let job_id = nanoid!();
    let internal_message = &Message::InternalWorkerQueueJob(Job {
        job_id,
        worker_id: workers_guard[*index_guard].clone(),
        request_id: job.request_id,
        guild_id: job.guild_id,
    });
    send_message(
        internal_message,
        config.kafka.kafka_topic.as_str(),
        producer,
    )
    .await;
    *index_guard += 1;
    if *index_guard == workers_guard.len() {
        *index_guard = 0;
    }
    Ok(())
}
