// Processing streaming jobs from kafka queue

// Parallelize with tokio

use flume::{Receiver, Sender};
use log::info;
use crate::config::Config;
use crate::utils::generic_connector::Message;
use crate::worker::webhook_handler::{VoiceDataRequestData, WebsocketInterconnect, WebsocketInterconnectType};

pub async fn process_job(message: Message, config: &Config, tx : Option<Sender<WebsocketInterconnect>>, rx : Option<Receiver<WebsocketInterconnect>>) {
    // Expect and Unwrap are fine here because if we panic it will only panic the thread so it should be fine in most cases
    let queue_job = message.queue_job_internal.expect("Internal job empty!");
    tx.unwrap().send(WebsocketInterconnect {
        com_type: WebsocketInterconnectType::VoiceDataRequest,
        voice_connect_data: None,
        voice_connect_request: Some(VoiceDataRequestData {
            guild_id: queue_job.guild_id.to_string(),
            channel_id: queue_job.voice_channel_id.to_string(),
            self_mute: false,
            self_deaf: false,
        })
    }).unwrap();
    info!("Job Processed!");
}