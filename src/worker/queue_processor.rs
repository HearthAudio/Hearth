use std::sync::Arc;
use songbird::Songbird;
use songbird::tracks::TrackHandle;
use tokio::sync::broadcast::{Receiver, Sender};
use crate::config::Config;
use crate::error_report;
use crate::utils::generic_connector::{DirectWorkerCommunication, DWCActionType, Message};
use serde::Deserialize;
use serde::Serialize;
use reqwest::Client as HttpClient;
use crate::worker::actions::channel_manager::{join_channel, leave_channel};
use crate::worker::actions::player::{play_direct_link, play_from_youtube};
use crate::worker::actions::track_manager::{force_stop_loop, pause_playback, resume_playback, set_playback_volume};

use super::actions::track_manager::{loop_indefinitely, loop_x_times, seek_to_position};

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
    let job_id = &queue_job.job_id;
    sender.send(ProcessorIPCData {
        action_type: ProcessorIncomingAction::Infrastructure(Infrastructure::SongbirdInstanceRequest),
        songbird: None,
        dwc: None,
        job_id: job_id.clone(),
        error_report: None,
    }).unwrap();
    let client = HttpClient::new(); // TEMP We should probs move this into an arc and share across jobs
    let mut manager : Option<Arc<Songbird>> = None;
    let mut track : Option<TrackHandle> = None;
    let mut ready = false;
    while let Ok(msg) = sender.subscribe().recv().await {
        if job_id == &msg.job_id {
            if ready == false {
                match msg.action_type {
                    ProcessorIncomingAction::Infrastructure(Infrastructure::SongbirdIncoming) => {
                        manager = msg.songbird;
                        ready = true;
                        // Join channel
                        let join = join_channel(&queue_job,&mut manager).await;
                        match join {
                            Ok(_) => {},
                            Err(e) => {
                                report_error(ErrorReport {
                                    error: e.to_string(),
                                    request_id: message.request_id.clone(),
                                    job_id: queue_job.job_id.clone()
                                })
                            }
                        }
                    },
                    _ => {}
                }
            } else {
                let dwc = msg.dwc.unwrap();
                match msg.action_type {
                    ProcessorIncomingAction::Actions(DWCActionType::LeaveChannel) => {
                       error_report!(leave_channel(&dwc,&mut manager).await,dwc.request_id.unwrap(),dwc.job_id.clone());
                    },
                    ProcessorIncomingAction::Actions(DWCActionType::LoopXTimes) => {
                        error_report!(loop_x_times(&track, dwc.loop_times).await,dwc.request_id.unwrap(),dwc.job_id.clone());
                    },
                    ProcessorIncomingAction::Actions(DWCActionType::ForceStopLoop) => {
                        error_report!(force_stop_loop(&track).await,dwc.request_id.unwrap(),dwc.job_id.clone());
                    },
                    ProcessorIncomingAction::Actions(DWCActionType::SeekToPosition) => {
                        error_report!(seek_to_position(&track, dwc.seek_position).await,dwc.request_id.unwrap(),dwc.job_id.clone());
                    }
                    ProcessorIncomingAction::Actions(DWCActionType::LoopForever) => {
                        error_report!(loop_indefinitely(&track).await,dwc.request_id.unwrap(),dwc.job_id.clone());
                    },
                    ProcessorIncomingAction::Actions(DWCActionType::PlayDirectLink) => {
                        error_report!(play_direct_link(&dwc,&mut manager,client.clone()).await,dwc.request_id.unwrap(),dwc.job_id.clone());
                    },
                    ProcessorIncomingAction::Actions(DWCActionType::PlayFromYoutube) => {
                        error_report!(play_from_youtube(&mut manager,&dwc,client.clone()).await,dwc.request_id.unwrap(),dwc.job_id.clone());
                    }
                    ProcessorIncomingAction::Actions(DWCActionType::PausePlayback) => {
                        error_report!(pause_playback(&track).await,dwc.request_id.unwrap(),dwc.job_id.clone());
                    },
                    ProcessorIncomingAction::Actions(DWCActionType::ResumePlayback) => {
                        error_report!(resume_playback(&track).await,dwc.request_id.unwrap(),dwc.job_id.clone());
                    }
                    ProcessorIncomingAction::Actions(DWCActionType::SetPlaybackVolume) => {
                        error_report!(set_playback_volume(&track,dwc.new_volume).await,dwc.request_id.unwrap(),dwc.job_id.clone());
                    }
                    _ => {}
                }
            }
        }
    }
}