// Processing streaming jobs from kafka queue

// Parallelize with tokio

use flume::{Receiver, Sender};
use log::info;
use crate::config::Config;
use crate::IPCWebsocketConnector;
use crate::utils::generic_connector::Message;
use crate::worker::webhook_handler::{VoiceDataRequestData, WebsocketInterconnect, WebsocketInterconnectType};

pub async fn process_job(message: Message, config: &Config,ipc: IPCWebsocketConnector) {
    // Expect and Unwrap are fine here because if we panic it will only panic the thread so it should be fine in most cases
    let queue_job = message.queue_job_internal.expect("Internal job empty!");
    ipc.voice_data_request_tx.send_async(WebsocketInterconnect {
        com_type: WebsocketInterconnectType::VoiceDataRequest,
        voice_connect_data: None,
        voice_connect_request: Some(VoiceDataRequestData {
            guild_id: queue_job.guild_id,
            channel_id: queue_job.voice_channel_id,
            self_mute: false,
            self_deaf: false,
        })
    }).await.unwrap();
    tokio::select! {
        flume_msg = ipc.voice_data_response_rx.recv_async() => {
            //TODO: Replace unwrap with match
            let parsed_msg : WebsocketInterconnect = flume_msg.unwrap();
            println!("RECV QP {:?}",parsed_msg);
            match parsed_msg.com_type {
                WebsocketInterconnectType::VoiceDataRequest => {},
                WebsocketInterconnectType::VoiceConnectDataResult => {
                    //TODO: Check if that result is for us or for another thread
                    info!("Job Processed!");
                }
            }
        }
    }
}