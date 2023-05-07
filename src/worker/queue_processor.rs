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
use serde::Deserialize;
use serde::Serialize;


#[derive(Clone,Debug)]
pub enum Infrastructure {
    SongbirdIncoming,
    SongbirdInstanceRequest,
    ErrorReport
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


pub async fn process_job(message: Message, _config: &Config, producer: &mut Producer, sender: Sender<ProcessorIPCData>,report_error: fn(ErrorReport)) {
    let queue_job = message.queue_job_internal.unwrap();
    let job_id = queue_job.job_id;
    sender.send(ProcessorIPCData {
        action_type: ProcessorIncomingAction::Infrastructure(Infrastructure::SongbirdInstanceRequest),
        songbird: None,
        dwc: None,
        job_id: job_id.clone(),
        error_report: None
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
                    },
                    _ => {}
                }
            } else {
                match msg.action_type {
                    ProcessorIncomingAction::Actions(DWCActionType::LeaveChannel) => {
                        let dwc = msg.dwc.unwrap();
                        manager.clone().unwrap().remove(GuildId(dwc.leave_channel_guild_id.unwrap().parse().unwrap())).await.unwrap();
                    }
                    ProcessorIncomingAction::Actions(DWCActionType::PlayDirectLink) => {
                        let dwc = msg.dwc.unwrap();
                        let handler_lock = manager.clone().unwrap().get(GuildId(1103499477962207332)).unwrap();
                        let mut handler = handler_lock.lock().await;
                        let source = url_source(dwc.play_audio_url.unwrap().as_str()).await;
                        match source {
                            Ok(res) => {
                                handler.play_source(res);
                            },
                            Err(e) => report_error(ErrorReport {
                                error: e.to_string(),
                                request_id: dwc.request_id,
                                job_id: dwc.job_id
                            })
                        }
                    },
                    ProcessorIncomingAction::Actions(DWCActionType::PlayFromYoutube) => {
                        let dwc = msg.dwc.unwrap();
                        let handler_lock = manager.clone().unwrap().get(GuildId(1103499477962207332)).unwrap();
                        let mut handler = handler_lock.lock().await;
                        let source = songbird::ytdl(dwc.play_audio_url.unwrap()).await.unwrap();
                        let internal_track = handler.play_source(source);
                        track = Some(internal_track);
                    }
                    ProcessorIncomingAction::Actions(DWCActionType::PausePlayback) => {
                        let _track = track.as_ref();
                        match _track {
                            Some(t) => {
                                let pause = t.pause();
                                match pause {
                                    Ok(_) => {},
                                    Err(e) => {
                                        report_error(ErrorReport {
                                            error: format!("Failed to pause with error: {}",e),
                                            request_id: msg.dwc.unwrap().request_id,
                                            job_id: msg.job_id
                                        })
                                    }
                                }
                            },
                            None => {
                                report_error(ErrorReport {
                                    error: "Track not found for pause playback".to_string(),
                                    request_id: msg.dwc.unwrap().request_id,
                                    job_id: msg.job_id
                                })
                            }
                        }
                    },
                    ProcessorIncomingAction::Actions(DWCActionType::ResumePlayback) => {
                        let _track = track.as_ref();
                        match _track {
                            Some(t) => {
                                let play = t.play();
                                match play {
                                    Ok(_) => {},
                                    Err(e) => {
                                        report_error(ErrorReport {
                                            error: format!("Failed to resume playback with error: {}",e),
                                            request_id: msg.dwc.unwrap().request_id,
                                            job_id: msg.job_id
                                        })
                                    }
                                }
                            },
                            None => {
                                report_error(ErrorReport {
                                    error: "Track not found for resume playback".to_string(),
                                    request_id: msg.dwc.unwrap().request_id,
                                    job_id: msg.job_id
                                })
                            }
                        }
                    }
                    ProcessorIncomingAction::Actions(DWCActionType::SetPlaybackVolume) => {
                        let _track = track.as_ref();
                        match _track {
                            Some(t) => {
                                let set_vol = t.set_volume(0.5);
                                match set_vol {
                                    Ok(_) => {},
                                    Err(e) => {
                                        report_error(ErrorReport {
                                            error: format!("Failed to set volume with error: {}",e),
                                            request_id: msg.dwc.unwrap().request_id,
                                            job_id: msg.job_id
                                        })
                                    }
                                }
                            },
                            None => {
                                report_error(ErrorReport {
                                    error: "Track not found for set volume".to_string(),
                                    request_id: msg.dwc.unwrap().request_id,
                                    job_id: msg.job_id
                                })
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
    }
}