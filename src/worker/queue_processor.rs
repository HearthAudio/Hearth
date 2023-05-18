use std::sync::Arc;

use songbird::Songbird;
use songbird::tracks::TrackHandle;
use tokio::sync::broadcast::{Receiver, Sender};
use crate::config::Config;
use crate::{error_report};
use hearth_interconnect::errors::ErrorReport;

use hearth_interconnect::worker_communication::{DirectWorkerCommunication, DWCActionType, Job};
use log::info;


use reqwest::Client as HttpClient;


use crate::worker::actions::channel_manager::{join_channel, leave_channel};
use crate::worker::actions::player::{play_direct_link, play_from_youtube};
use crate::worker::actions::track_manager::{force_stop_loop, pause_playback, resume_playback, set_playback_volume};
use crate::worker::constants::DEFAULT_JOB_EXPIRATION_TIME;

use crate::worker::helpers::get_unix_timestamp_as_seconds;
use super::actions::metadata::get_metadata;
use super::actions::track_manager::{loop_indefinitely, loop_x_times, seek_to_position};

#[derive(Clone,Debug)]
pub enum Infrastructure {
    CheckTime
}

#[derive(Clone,Debug)]
pub enum ProcessorIncomingAction {
    Infrastructure(Infrastructure),
    Actions(DWCActionType)
}

#[derive(Clone,Debug,PartialEq)]
pub enum JobID {
    Global(),
    Specific(String)
}

impl JobID {
    pub fn to_string(&self) -> String {
        // We don't want to disclose the secret
        if let JobID::Specific(s) = self {
            format!("{}",s);
        }
        format!("GLOBAL")
    }
}

#[derive(Clone,Debug)]
pub struct ProcessorIPCData {
    pub action_type: ProcessorIncomingAction,
    pub songbird: Option<Arc<Songbird>>,
    pub dwc: Option<DirectWorkerCommunication>,
    pub error_report: Option<ErrorReport>,
    pub job_id: JobID,
}


pub struct ProcessorIPC {
    pub sender: Arc<Sender<ProcessorIPCData>>,
    pub receiver: Receiver<ProcessorIPCData>
}


pub async fn process_job(job: Job, config: &Config, sender: Arc<Sender<ProcessorIPCData>>, report_error: fn(ErrorReport,&Config), mut manager: Option<Arc<Songbird>>) {
    let job_id = JobID::Specific(job.job_id.clone());
    let global_job_id = JobID::Global();
    let client = HttpClient::new();



    // Start core
    let mut track: Option<TrackHandle> = None;
    let start_time = get_unix_timestamp_as_seconds();
    info!("Worker started");
    while let Ok(msg) = sender.subscribe().recv().await {
        if job_id == msg.job_id || msg.job_id == global_job_id {
            let dwc : Option<DirectWorkerCommunication> = msg.dwc;
            match msg.action_type {
                ProcessorIncomingAction::Infrastructure(Infrastructure::CheckTime) => {
                    // If this job has been running for more than designated time break it
                    let current_time = get_unix_timestamp_as_seconds();
                    let time_change = current_time - start_time;
                    if time_change > config.config.job_expiration_time_seconds.unwrap_or(DEFAULT_JOB_EXPIRATION_TIME) {
                        info!("Killing JOB: {} due to expiration after: {} hours",job_id.to_string(),(time_change / 60) / 60);
                        break;
                    }
                },
                ProcessorIncomingAction::Actions(DWCActionType::JoinChannel) => {
                    // Join channel
                    let dwc = dwc.expect("This should never happen. Because this is a DWC type and is parsed previously.");
                    let join = join_channel(dwc.guild_id.unwrap(), dwc.voice_channel_id.unwrap(), job_id.to_string(), dwc.request_id.unwrap(), &mut manager, report_error, config.clone()).await;
                    let _ = error_report!(join,job.request_id.clone(),job_id.to_string(),config);
                }
                ProcessorIncomingAction::Actions(DWCActionType::LeaveChannel) => {
                    let dwc = dwc.expect("This should never happen. Because this is a DWC type and is parsed previously.");
                    let _ = error_report!(leave_channel(&dwc,&mut manager).await,dwc.request_id.unwrap(),dwc.job_id.clone(),config);
                },
                ProcessorIncomingAction::Actions(DWCActionType::LoopXTimes) => {
                    let dwc = dwc.expect("This should never happen. Because this is a DWC type and is parsed previously.");
                    let _ = error_report!(loop_x_times(&track, dwc.loop_times).await,dwc.request_id.unwrap(),dwc.job_id.clone(),config);
                },
                ProcessorIncomingAction::Actions(DWCActionType::ForceStopLoop) => {
                    let dwc = dwc.expect("This should never happen. Because this is a DWC type and is parsed previously.");
                    let _ = error_report!(force_stop_loop(&track).await,dwc.request_id.unwrap(),dwc.job_id.clone(),config);
                },
                ProcessorIncomingAction::Actions(DWCActionType::SeekToPosition) => {
                    let dwc = dwc.expect("This should never happen. Because this is a DWC type and is parsed previously.");
                    let _ = error_report!(seek_to_position(&track, dwc.seek_position).await,dwc.request_id.unwrap(),dwc.job_id.clone(),config);
                }
                ProcessorIncomingAction::Actions(DWCActionType::LoopForever) => {
                    let dwc = dwc.expect("This should never happen. Because this is a DWC type and is parsed previously.");
                    let _ = error_report!(loop_indefinitely(&track).await,dwc.request_id.unwrap(),dwc.job_id.clone(),config);
                },
                ProcessorIncomingAction::Actions(DWCActionType::PlayDirectLink) => {
                    let dwc = dwc.expect("This should never happen. Because this is a DWC type and is parsed previously.");
                    track = error_report!(play_direct_link(&dwc,&mut manager,client.clone()).await,dwc.request_id.unwrap(),dwc.job_id.clone(),config);
                },
                ProcessorIncomingAction::Actions(DWCActionType::PlayFromYoutube) => {
                    let dwc = dwc.expect("This should never happen. Because this is a DWC type and is parsed previously.");
                    track = error_report!(play_from_youtube(&mut manager,&dwc,client.clone()).await,dwc.request_id.unwrap(),dwc.job_id.clone(),config);
                }
                ProcessorIncomingAction::Actions(DWCActionType::PausePlayback) => {
                    let dwc = dwc.expect("This should never happen. Because this is a DWC type and is parsed previously.");
                    let _ = error_report!(pause_playback(&track).await,dwc.request_id.unwrap(),dwc.job_id.clone(),config);
                },
                ProcessorIncomingAction::Actions(DWCActionType::ResumePlayback) => {
                    let dwc = dwc.expect("This should never happen. Because this is a DWC type and is parsed previously.");
                    let _ = error_report!(resume_playback(&track).await,dwc.request_id.unwrap(),dwc.job_id.clone(),config);
                }
                ProcessorIncomingAction::Actions(DWCActionType::SetPlaybackVolume) => {
                    let dwc = dwc.expect("This should never happen. Because this is a DWC type and is parsed previously.");
                    let _ = error_report!(set_playback_volume(&track,dwc.new_volume).await,dwc.request_id.unwrap(),dwc.job_id.clone(),config);
                }
                ProcessorIncomingAction::Actions(DWCActionType::GetMetaData) => {
                    let dwc = dwc.expect("This should never happen. Because this is a DWC type and is parsed previously.");
                    let _ = error_report!(get_metadata(&track,config,dwc.request_id.clone().unwrap(),dwc.job_id.clone()).await,dwc.request_id.unwrap(),dwc.job_id.clone(),config);
                }
                _ => {}
            }
        }
    }
}