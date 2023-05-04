use std::sync::Arc;
use std::time::Instant;
use log::info;
use serenity::model::id::{ChannelId, GuildId};
use songbird::Songbird;
use tokio::sync::broadcast::{Receiver, Sender};
use crate::config::Config;
use crate::utils::generic_connector::Message;
use crate::worker::songbird_handler::{SongbirdRequestData};

#[derive(Clone,Debug)]
pub enum ProcessorIncomingAction {
    SongbirdIncoming,
    LeaveChannel,
    SongbirdInstanceRequest
}

#[derive(Clone,Debug)]
pub struct LeaveAction {
    pub guild_id: u64
}

#[derive(Clone,Debug)]
pub struct ProcessorIPCData {
    pub action: ProcessorIncomingAction,
    pub job_id: String,
    pub songbird: Option<Arc<Songbird>>,
    pub leave_action: Option<LeaveAction>,
}


pub struct ProcessorIPC {
    pub sender: Sender<ProcessorIPCData>,
    pub receiver: Receiver<ProcessorIPCData>
}


pub async fn process_job(message: Message, config: &Config, sender: Sender<ProcessorIPCData>) {
    let queue_job = message.queue_job_internal.unwrap();
    let job_id = queue_job.job_id;
    sender.send(ProcessorIPCData {
        action: ProcessorIncomingAction::SongbirdInstanceRequest,
        job_id: job_id.clone(),
        songbird: None,
        leave_action: None,
    }).unwrap();
    let mut manager : Option<Arc<Songbird>> = None;
    while let Ok(msg) = sender.subscribe().recv().await {
        if job_id == msg.job_id {
            match msg.action {
                ProcessorIncomingAction::SongbirdIncoming => {
                    manager = msg.songbird;
                    let _handler = manager.clone().unwrap().join(GuildId(queue_job.guild_id.parse().unwrap()), ChannelId(queue_job.voice_channel_id.parse().unwrap())).await;
                    info!("Processed Job!");
                },
                ProcessorIncomingAction::LeaveChannel => {
                    manager.clone().unwrap().remove(GuildId(msg.leave_action.unwrap().guild_id)).await.unwrap();
                }
                _ => {}
            }
        }
    }
}