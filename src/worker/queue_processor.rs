use std::sync::Arc;
use songbird::Songbird;
use songbird::tracks::TrackHandle;
use tokio::sync::broadcast::{Receiver, Sender};
use crate::config::Config;
use crate::error_report;
use hearth_interconnect::errors::ErrorReport;
use hearth_interconnect::messages::Message;
use hearth_interconnect::worker_communication::{DirectWorkerCommunication, DWCActionType, Job};
use serde::Deserialize;
use serde::Serialize;
use reqwest::Client as HttpClient;
use songbird::input::AuxMetadata;
use crate::worker::actions::channel_manager::{join_channel, leave_channel};
use crate::worker::actions::player::{play_direct_link, play_from_youtube, PlaybackResult};
use crate::worker::actions::track_manager::{force_stop_loop, send_metadata, pause_playback, resume_playback, set_playback_volume};

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


pub async fn process_job(job: Job, config: &Config, sender: Sender<ProcessorIPCData>,report_error: fn(ErrorReport,&Config)) {
    let job_id = &job.job_id;
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
    let mut metadata : Option<AuxMetadata> = None;
    let mut ready = false;
    while let Ok(msg) = sender.subscribe().recv().await {
        if job_id == &msg.job_id {
            if !ready {
                if let ProcessorIncomingAction::Infrastructure(Infrastructure::SongbirdIncoming) = msg.action_type {
                    manager = msg.songbird;
                    ready = true;
                    // Join channel
                    let job_id = job.job_id.clone();
                    let join = join_channel(&job,job.request_id.clone(),&mut manager,report_error,config.clone()).await;
                    let _ = error_report!(join,msg.dwc.unwrap().request_id.unwrap(),job_id,config);
                }
            } else {
                let dwc = msg.dwc.unwrap();
                match msg.action_type {
                    ProcessorIncomingAction::Actions(DWCActionType::LeaveChannel) => {
                        let _ = error_report!(leave_channel(&dwc,&mut manager).await,dwc.request_id.unwrap(),dwc.job_id.clone(),config);
                    },
                    ProcessorIncomingAction::Actions(DWCActionType::LoopXTimes) => {
                        let _ = error_report!(loop_x_times(&track, dwc.loop_times).await,dwc.request_id.unwrap(),dwc.job_id.clone(),config);
                    },
                    ProcessorIncomingAction::Actions(DWCActionType::ForceStopLoop) => {
                        let _ = error_report!(force_stop_loop(&track).await,dwc.request_id.unwrap(),dwc.job_id.clone(),config);
                    },
                    ProcessorIncomingAction::Actions(DWCActionType::SeekToPosition) => {
                        let _ = error_report!(seek_to_position(&track, dwc.seek_position).await,dwc.request_id.unwrap(),dwc.job_id.clone(),config);
                    }
                    ProcessorIncomingAction::Actions(DWCActionType::LoopForever) => {
                        let _ = error_report!(loop_indefinitely(&track).await,dwc.request_id.unwrap(),dwc.job_id.clone(),config);
                    },
                    ProcessorIncomingAction::Actions(DWCActionType::PlayDirectLink) => {
                        let res = error_report!(play_direct_link(&dwc,&mut manager,client.clone()).await,dwc.request_id.unwrap(),dwc.job_id.clone(),config);
                    },
                    ProcessorIncomingAction::Actions(DWCActionType::PlayFromYoutube) => {
                        let res = error_report!(play_from_youtube(&mut manager,&dwc,client.clone()).await,dwc.request_id.unwrap(),dwc.job_id.clone(),config);
                        if let Some(r) = res {
                            track = Some(r.track_handle);
                            metadata = Some(r.metadata);
                        }
                    }
                    ProcessorIncomingAction::Actions(DWCActionType::PausePlayback) => {
                        let _ = error_report!(pause_playback(&track).await,dwc.request_id.unwrap(),dwc.job_id.clone(),config);
                    },
                    ProcessorIncomingAction::Actions(DWCActionType::ResumePlayback) => {
                        let _ = error_report!(resume_playback(&track).await,dwc.request_id.unwrap(),dwc.job_id.clone(),config);
                    }
                    ProcessorIncomingAction::Actions(DWCActionType::SetPlaybackVolume) => {
                        let _ = error_report!(set_playback_volume(&track,dwc.new_volume).await,dwc.request_id.unwrap(),dwc.job_id.clone(),config);
                    }
                    ProcessorIncomingAction::Actions(DWCActionType::GetMetaData) => {
                        let _ = error_report!(send_metadata(&track).await,dwc.request_id.unwrap(),dwc.job_id.clone(),config);
                    }
                    _ => {}
                }
            }
        }
    }
}