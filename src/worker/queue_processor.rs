use std::fs;
use std::fs::File;
use std::sync::Arc;
use std::time::Instant;
use log::info;
use nanoid::nanoid;
use reqwest::Client;
use serenity::model::id::{ChannelId, GuildId};
use songbird::Songbird;
use tokio::sync::broadcast::{Receiver, Sender};
use crate::config::Config;
use crate::utils::generic_connector::{DWCActionType, Message};
use crate::worker::songbird_handler::{SongbirdRequestData};
use bytes::{Buf, Bytes};
use crate::worker::sources::url::create_url_input;


#[derive(Clone,Debug)]
pub enum Infrastructure {
    SongbirdIncoming,
    SongbirdInstanceRequest
}

#[derive(Clone,Debug)]
pub enum ProcessorIncomingAction {
    Infrastructure(Infrastructure),
    Actions(DWCActionType)
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
        action: ProcessorIncomingAction::Infrastructure(Infrastructure::SongbirdInstanceRequest),
        job_id: job_id.clone(),
        songbird: None,
        leave_action: None,
    }).unwrap();
    let mut manager : Option<Arc<Songbird>> = None;
    while let Ok(msg) = sender.subscribe().recv().await {
        if job_id == msg.job_id {
            match msg.action {
                ProcessorIncomingAction::Infrastructure(Infrastructure::SongbirdIncoming) => {
                    manager = msg.songbird;
                    let _handler = manager.clone().unwrap().join(GuildId(queue_job.guild_id.parse().unwrap()), ChannelId(queue_job.voice_channel_id.parse().unwrap())).await;
                },
                ProcessorIncomingAction::Actions(DWCActionType::LeaveChannel) => {
                    manager.clone().unwrap().remove(GuildId(msg.leave_action.unwrap().guild_id)).await.unwrap();
                }
                ProcessorIncomingAction::Actions(DWCActionType::PlayDirectLink) => {
                    let handler_lock = manager.clone().unwrap().get(GuildId(1103499477962207332)).unwrap();
                    let mut handler = handler_lock.lock().await;
                    let source = create_url_input("https://firmware-repo-esp32.s3.us-west-1.amazonaws.com/PinkPanther30.wav").await;
                    handler.play_source(source);
                }
                _ => {}
            }
        }
    }
}