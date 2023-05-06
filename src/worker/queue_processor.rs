

use std::sync::Arc;




use serenity::model::id::{ChannelId, GuildId};
use songbird::Songbird;
use tokio::sync::broadcast::{Receiver, Sender};
use crate::config::Config;
use crate::utils::generic_connector::{DirectWorkerCommunication, DWCActionType, Message};
use crate::worker::sources::url::url_source;


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
pub struct PlayAudioAction {
    pub url: String
}

#[derive(Clone,Debug)]
pub struct ProcessorIPCData {
    pub action_type: ProcessorIncomingAction,
    pub songbird: Option<Arc<Songbird>>,
    pub dwc: Option<DirectWorkerCommunication>,
    pub job_id: String,
}


pub struct ProcessorIPC {
    pub sender: Sender<ProcessorIPCData>,
    pub receiver: Receiver<ProcessorIPCData>
}


pub async fn process_job(message: Message, _config: &Config, sender: Sender<ProcessorIPCData>) {
    let queue_job = message.queue_job_internal.unwrap();
    let job_id = queue_job.job_id;
    sender.send(ProcessorIPCData {
        action_type: ProcessorIncomingAction::Infrastructure(Infrastructure::SongbirdInstanceRequest),
        songbird: None,
        dwc: None,
        job_id: job_id.clone()
    }).unwrap();
    let mut manager : Option<Arc<Songbird>> = None;
    while let Ok(msg) = sender.subscribe().recv().await {
        if job_id == msg.job_id {
            match msg.action_type {
                ProcessorIncomingAction::Infrastructure(Infrastructure::SongbirdIncoming) => {
                    manager = msg.songbird;
                    let _handler = manager.clone().unwrap().join(GuildId(queue_job.guild_id.parse().unwrap()), ChannelId(queue_job.voice_channel_id.parse().unwrap())).await;
                },
                ProcessorIncomingAction::Actions(DWCActionType::LeaveChannel) => {
                    let dwc = msg.dwc.unwrap();
                    manager.clone().unwrap().remove(GuildId(dwc.leave_channel_guild_id.unwrap().parse().unwrap())).await.unwrap();
                }
                ProcessorIncomingAction::Actions(DWCActionType::PlayDirectLink) => {
                    let dwc = msg.dwc.unwrap();
                    let handler_lock = manager.clone().unwrap().get(GuildId(1103499477962207332)).unwrap();
                    let mut handler = handler_lock.lock().await;
                    let source = url_source(dwc.play_audio_url.unwrap()).await;
                    handler.play_source(source);
                },
                ProcessorIncomingAction::Actions(DWCActionType::PlayFromYoutube) => {
                    let dwc = msg.dwc.unwrap();
                    let handler_lock = manager.clone().unwrap().get(GuildId(1103499477962207332)).unwrap();
                    let mut handler = handler_lock.lock().await;
                    let source = songbird::ytdl(dwc.play_audio_url.unwrap()).await.unwrap();
                    handler.play_source(source);
                }
                _ => {}
            }
        }
    }
}