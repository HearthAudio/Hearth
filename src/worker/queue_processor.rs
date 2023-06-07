use std::fmt;
use std::sync::Arc;
use songbird::Songbird;
use songbird::tracks::TrackHandle;
use tokio::sync::broadcast::{Receiver, Sender};
use crate::config::Config;
use crate::{error_report};
use hearth_interconnect::errors::ErrorReport;
use hearth_interconnect::messages::{ExternalQueueJobResponse, JobExpired, Message};
use hearth_interconnect::worker_communication::{DirectWorkerCommunication, DWCActionType, Job};
use log::info;
use reqwest::Client as HttpClient;
use crate::worker::actions::channel_manager::{join_channel, leave_channel};
use crate::worker::actions::player::{play_direct_link, play_from_youtube};
use crate::worker::actions::track_manager::{force_stop_loop, pause_playback, resume_playback, set_playback_volume};
use crate::worker::connector::{send_message, WORKER_PRODUCER};
use crate::worker::constants::{DEFAULT_JOB_EXPIRATION_TIME, DEFAULT_JOB_EXPIRATION_TIME_NOT_PLAYING};
use crate::worker::helpers::get_unix_timestamp_as_seconds;
use super::actions::metadata::get_metadata;
use super::actions::track_manager::{loop_indefinitely, loop_x_times, seek_to_position};

#[derive(Clone,Debug)]
pub enum Infrastructure {
    CheckTime,
    TrackEnded
}

#[derive(Clone,Debug)]
pub enum ProcessorIncomingAction {
    Infrastructure(Infrastructure),
    Actions(DWCActionType)
}

#[derive(Clone,Debug,PartialEq,Eq,Hash)]
pub enum JobID {
    Global(),
    Specific(String)
}

impl fmt::Display for JobID {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if let JobID::Specific(s) = self {
            return write!(f, "{}",s);
        }
        write!(f, "GLOBAL")
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

async fn notify_expiration(guild_id: String, job_id: String,config: &Config) {
    let mut px = WORKER_PRODUCER.get().unwrap().lock().await;
    let p = px.as_mut();

    send_message(&Message::ExternalJobExpired(JobExpired {
        guild_id,
        job_id
    }),&config.kafka.kafka_topic,&mut p.unwrap()).await;
}

pub async fn process_job(job: Job, config: &Config, sender: Arc<Sender<ProcessorIPCData>>, report_error: fn(ErrorReport,&Config), mut manager: Option<Arc<Songbird>>) {
    let job_id = JobID::Specific(job.job_id.clone());
    let global_job_id = JobID::Global();
    let guild_id = job.guild_id.clone();
    let client = HttpClient::new();

    //
    let mut track: Option<TrackHandle> = None;
    let start_time = get_unix_timestamp_as_seconds();
    let mut last_play_end_time : Option<u64> = None;
    let mut is_playing = false;

    // Send Queue Job Response
    { // Scoped to release producer mutex
        let mut px = WORKER_PRODUCER.get().unwrap().lock().await;
        let p = px.as_mut();

        send_message(&Message::ExternalQueueJobResponse(ExternalQueueJobResponse {
            job_id: job_id.to_string(),
            worker_id: config.config.worker_id.as_ref().unwrap().clone(),
            guild_id: job.guild_id,
        }), config.kafka.kafka_topic.as_str(), &mut *p.unwrap()).await;
    }

    // Start core
    info!("Worker started");
    while let Ok(msg) = sender.subscribe().recv().await {
        if job_id == msg.job_id || msg.job_id == global_job_id {
            let dwc : Option<DirectWorkerCommunication> = msg.dwc;
            match msg.action_type {
                ProcessorIncomingAction::Infrastructure(Infrastructure::CheckTime) => {
                    // If this job has been running for more than designated time break it
                    let current_time = get_unix_timestamp_as_seconds();
                    let time_change = current_time - start_time;

                    // If nothing is playing use different end time
                    if !is_playing  && current_time - last_play_end_time.unwrap() > config.config.job_expiration_time_seconds_not_playing.unwrap_or(DEFAULT_JOB_EXPIRATION_TIME_NOT_PLAYING) {
                        info!("Killing JOB: {} due to expiration after: {} hours while not playing",job_id.to_string(),(time_change / 60) / 60);
                        notify_expiration(guild_id,job_id.to_string(),config).await;
                        break;
                    }

                    // If something is playing use different end time
                    if time_change > config.config.job_expiration_time_seconds.unwrap_or(DEFAULT_JOB_EXPIRATION_TIME) && is_playing {
                        info!("Killing JOB: {} due to expiration after: {} hours while playing",job_id.to_string(),(time_change / 60) / 60);
                        notify_expiration(guild_id,job_id.to_string(),config).await;
                        break;
                    }
                },
                ProcessorIncomingAction::Actions(DWCActionType::JoinChannel) => {
                    // Join channel
                    let dwc = dwc.expect("This should never happen. Because this is a DWC type and is parsed previously.");
                    let join = join_channel(dwc.guild_id, dwc.voice_channel_id.unwrap(), job_id.clone(), dwc.request_id.unwrap(), &mut manager, report_error, config.clone(),sender.clone()).await;
                    error_report!(join,job.request_id.clone(),job_id.to_string(),guild_id.clone(),config);
                }
                ProcessorIncomingAction::Actions(DWCActionType::LeaveChannel) => {
                    let dwc = dwc.expect("This should never happen. Because this is a DWC type and is parsed previously.");
                    error_report!(leave_channel(&dwc,&mut manager).await,dwc.request_id.unwrap(),dwc.job_id.clone(),guild_id.clone(),config);
                    track = None;
                    is_playing = false;
                    last_play_end_time = Some(get_unix_timestamp_as_seconds());
                },
                ProcessorIncomingAction::Actions(DWCActionType::LoopXTimes) => {
                    let dwc = dwc.expect("This should never happen. Because this is a DWC type and is parsed previously.");
                    error_report!(loop_x_times(&track, dwc.loop_times).await,dwc.request_id.unwrap(),dwc.job_id.clone(),guild_id.clone(),config);
                },
                ProcessorIncomingAction::Actions(DWCActionType::ForceStopLoop) => {
                    let dwc = dwc.expect("This should never happen. Because this is a DWC type and is parsed previously.");
                    error_report!(force_stop_loop(&track).await,dwc.request_id.unwrap(),dwc.job_id.clone(),guild_id.clone(),config);
                },
                ProcessorIncomingAction::Actions(DWCActionType::SeekToPosition) => {
                    let dwc = dwc.expect("This should never happen. Because this is a DWC type and is parsed previously.");
                    error_report!(seek_to_position(&track, dwc.seek_position).await,dwc.request_id.unwrap(),dwc.job_id.clone(),guild_id.clone(),config);
                }
                ProcessorIncomingAction::Actions(DWCActionType::LoopForever) => {
                    let dwc = dwc.expect("This should never happen. Because this is a DWC type and is parsed previously.");
                    error_report!(loop_indefinitely(&track).await,dwc.request_id.unwrap(),dwc.job_id.clone(),guild_id.clone(),config);
                },
                ProcessorIncomingAction::Actions(DWCActionType::PlayDirectLink) => {
                    let dwc = dwc.expect("This should never happen. Because this is a DWC type and is parsed previously.");
                    // Make sure we are not already playing something on this handler
                    if !is_playing {
                        track = error_report!(play_direct_link(&dwc,&mut manager,client.clone()).await,dwc.request_id.unwrap(),dwc.job_id.clone(),guild_id.clone(),config);
                        is_playing = true;
                    } else {
                        report_error(ErrorReport {
                            error: "Already playing!".to_string(),
                            request_id: dwc.request_id.unwrap(),
                            job_id: job_id.to_string(),
                            guild_id: guild_id.clone()
                        },config);
                    }
                },
                ProcessorIncomingAction::Actions(DWCActionType::PlayFromYoutube) => {
                    let dwc = dwc.expect("This should never happen. Because this is a DWC type and is parsed previously.");
                    // Make sure we are not already playing something on this handler
                    if !is_playing {
                        track = error_report!(play_from_youtube(&mut manager,&dwc,client.clone()).await,dwc.request_id.unwrap(),dwc.job_id.clone(),guild_id.clone(),config);
                        is_playing = true;
                    } else {
                        report_error(ErrorReport {
                            error: "Already playing!".to_string(),
                            request_id: dwc.request_id.unwrap(),
                            job_id: job_id.to_string(),
                            guild_id: guild_id.clone()
                        },config);
                    }
                }
                ProcessorIncomingAction::Actions(DWCActionType::PausePlayback) => {
                    let dwc = dwc.expect("This should never happen. Because this is a DWC type and is parsed previously.");
                    error_report!(pause_playback(&track).await,dwc.request_id.unwrap(),dwc.job_id.clone(),guild_id.clone(),config);
                    is_playing = false;
                },
                ProcessorIncomingAction::Actions(DWCActionType::ResumePlayback) => {
                    let dwc = dwc.expect("This should never happen. Because this is a DWC type and is parsed previously.");
                    error_report!(resume_playback(&track).await,dwc.request_id.unwrap(),dwc.job_id.clone(),guild_id.clone(),config);
                    is_playing = true;
                }
                ProcessorIncomingAction::Actions(DWCActionType::SetPlaybackVolume) => {
                    let dwc = dwc.expect("This should never happen. Because this is a DWC type and is parsed previously.");
                    error_report!(set_playback_volume(&track,dwc.new_volume).await,dwc.request_id.unwrap(),dwc.job_id.clone(),guild_id.clone(),config);
                }
                ProcessorIncomingAction::Actions(DWCActionType::GetMetaData) => {
                    let dwc = dwc.expect("This should never happen. Because this is a DWC type and is parsed previously.");
                    error_report!(get_metadata(&track,config,dwc.request_id.clone().unwrap(),dwc.job_id.clone(), dwc.guild_id.clone()).await,dwc.request_id.unwrap(),dwc.job_id.clone(),dwc.guild_id,config);
                },
                ProcessorIncomingAction::Infrastructure(Infrastructure::TrackEnded) => {
                    info!("Track ended on Job: {}",job_id.to_string());
                    is_playing = false;
                    track = None;
                }
                _ => {}
            }
        }
    }
}