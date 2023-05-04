use std::time::Instant;
use log::info;
use serenity::model::id::{ChannelId, GuildId};
use crate::config::Config;
use crate::utils::generic_connector::Message;
use crate::worker::songbird_handler::{SongbirdIPC, SongbirdRequestData};

pub async fn process_job(message: Message,config: &Config,songbird_ipc: &SongbirdIPC) {
    let queue_job = message.queue_job_internal.unwrap();
    songbird_ipc.tx_songbird_request.send_async(false).await.unwrap(); // bool value does not matter
    while let Ok(msg) = songbird_ipc.rx_songbird_result.recv_async().await {
        let manager = msg.unwrap();
        let _handler = manager.join(GuildId(queue_job.guild_id.parse().unwrap()), ChannelId(queue_job.voice_channel_id.parse().unwrap())).await;
        info!("Processed Job!");
        break;
    }
}