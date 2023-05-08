use std::sync::Arc;
use kafka::producer::Producer;
use log::error;
use serenity::model::id::{ChannelId, GuildId};
use songbird::Songbird;
use songbird::tracks::TrackHandle;
use tokio::sync::broadcast::{Receiver, Sender};
use crate::config::Config;
use crate::utils::generic_connector::{DirectWorkerCommunication, DWCActionType, Message};
use crate::worker::sources::url::url_source;
use crate::actions::*;
use serde::Deserialize;
use serde::Serialize;
use serenity::cache::Cache;
use serenity::Client;
use snafu::whatever;
use crate::worker::actions::channel_manager::leave_channel;
use crate::worker::actions::player::{play_direct_link, play_from_youtube};
use crate::worker::actions::track_manager::{pause_playback, resume_playback, set_playback_volume};

#[derive(Clone,Debug)]
pub enum Infrastructure {
    SongbirdIncoming,
    SongbirdInstanceRequest,
    ErrorReport,
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

#[derive(Deserialize,Debug,Serialize,Clone)]
pub struct ErrorReport {
    pub error: String,
    pub request_id: String,
    pub job_id: String
}

#[derive(Clone,Debug)]
pub struct ProcessorIPCData {
    pub action_type: ProcessorIncomingAction,
    pub songbird: Option<Arc<Songbird>>,
    pub dwc: Option<DirectWorkerCommunication>,
    pub error_report: Option<ErrorReport>,
    pub job_id: String,
}


pub struct ProcessorIPC {
    pub sender: Sender<ProcessorIPCData>,
    pub receiver: Receiver<ProcessorIPCData>
}


pub async fn process_job(message: Message, _config: &Config, sender: Sender<ProcessorIPCData>,report_error: fn(ErrorReport)) {
    let queue_job = message.queue_job_internal.unwrap();
    let job_id = queue_job.job_id;
    sender.send(ProcessorIPCData {
        action_type: ProcessorIncomingAction::Infrastructure(Infrastructure::SongbirdInstanceRequest),
        songbird: None,
        dwc: None,
        job_id: job_id.clone(),
        error_report: None,
    }).unwrap();
    let mut manager : Option<Arc<Songbird>> = None;
    let mut track : Option<TrackHandle> = None;
    let mut ready = false;
    while let Ok(msg) = sender.subscribe().recv().await {
        if job_id == msg.job_id {
            if ready == false {
                match msg.action_type {
                    ProcessorIncomingAction::Infrastructure(Infrastructure::SongbirdIncoming) => {
                        manager = msg.songbird;
                        ready = true;
                        // Join channel
                        manager.as_mut().unwrap().join(GuildId(queue_job.guild_id.clone().parse().unwrap()), ChannelId(queue_job.voice_channel_id.clone().parse().unwrap())).await;
                    },
                    _ => {}
                }
            } else {
                let dwc = msg.dwc.unwrap();
                match msg.action_type {
                    ProcessorIncomingAction::Actions(DWCActionType::LeaveChannel) => {
                       leave_channel(&dwc,&mut manager).await;
                    }
                    ProcessorIncomingAction::Actions(DWCActionType::PlayDirectLink) => {
                        let play = play_direct_link(&dwc,&mut manager).await;
                        match play {
                            Ok(t) => {
                                track = Some(t);
                            },
                            Err(e) => {
                                report_error(ErrorReport {
                                    error: e.to_string(),
                                    request_id: dwc.request_id.unwrap(),
                                    job_id: dwc.job_id.clone()
                                })
                            }
                        }
                    },
                    ProcessorIncomingAction::Actions(DWCActionType::PlayFromYoutube) => {
                        let youtube = play_from_youtube(&mut manager,&dwc).await;
                        match youtube {
                            Ok(t) => {
                                track = Some(t);
                            },
                            Err(e) => report_error(ErrorReport {
                                error: e.to_string(),
                                request_id: dwc.request_id.unwrap(),
                                job_id: dwc.job_id.clone()
                            })
                        }
                    }
                    ProcessorIncomingAction::Actions(DWCActionType::PausePlayback) => {
                        let pause = pause_playback(&track).await;
                        match pause {
                            Ok(_) => {},
                            Err(e) => report_error(ErrorReport {
                                error: e.to_string(),
                                request_id: dwc.request_id.unwrap(),
                                job_id: msg.job_id
                            })
                        }
                    },
                    ProcessorIncomingAction::Actions(DWCActionType::ResumePlayback) => {
                        let play = resume_playback(&track).await;
                        match play {
                            Ok(_) => {},
                            Err(e) => report_error(ErrorReport {
                                error: e.to_string(),
                                request_id: dwc.request_id.unwrap(),
                                job_id: msg.job_id
                            })
                        }
                    }
                    ProcessorIncomingAction::Actions(DWCActionType::SetPlaybackVolume) => {
                        let vol = set_playback_volume(&track,dwc.new_volume).await;
                        match vol {
                            Ok(_) => {},
                            Err(e) => report_error(ErrorReport {
                                error: e.to_string(),
                                request_id: dwc.request_id.unwrap(),
                                job_id: msg.job_id
                            })
                        }
                    }
                    _ => {}
                }
            }
        }
    }
}