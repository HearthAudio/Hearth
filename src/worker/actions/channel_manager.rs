use std::sync::Arc;
use hearth_interconnect::errors::ErrorReport;
use hearth_interconnect::worker_communication::{DirectWorkerCommunication};
use songbird::id::GuildId;
use songbird::id::ChannelId;
use songbird::{Event, EventContext, Songbird};
use songbird::events::{EventHandler as VoiceEventHandler,TrackEvent};
use serenity::async_trait;
use crate::config::Config;
use std::fmt;
use anyhow::{Context, Result};
use log::error;
use tokio::sync::broadcast::Sender;
use crate::worker::queue_processor::{Infrastructure, JobID, ProcessorIncomingAction, ProcessorIPCData};

pub enum ChannelControlError {
    GuildIDParsingFailed,
    ChannelIDParsingFailed,
    ManagerAcquisitionFailed,
    ChannelLeaveFailed,
}

impl fmt::Display for ChannelControlError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ChannelControlError::GuildIDParsingFailed => write!(f, "Guild ID Parsing Failed"),
            ChannelControlError::ChannelIDParsingFailed =>  write!(f, "Channel ID Parsing Failed"),
            ChannelControlError::ManagerAcquisitionFailed =>  write!(f, "Failed to acquire manager"),
            ChannelControlError::ChannelLeaveFailed => write!(f, "Failed to leave channel"),
        }
    }
}

pub async fn leave_channel(dwc: &DirectWorkerCommunication, manager: &mut Option<Arc<Songbird>>) -> Result<()> {
    manager.as_mut().context(ChannelControlError::ManagerAcquisitionFailed.to_string())?.remove(GuildId(dwc.guild_id.parse().context(ChannelControlError::GuildIDParsingFailed.to_string())?)).await.context(ChannelControlError::ChannelLeaveFailed.to_string())?;
    Ok(())
}

struct TrackErrorNotifier {
    error_reporter: fn(ErrorReport,&Config),
    request_id: String,
    guild_id: String,
    job_id: JobID,
    config: Config
}

struct TrackEndNotifier {
    job_id: JobID,
    tx: Arc<Sender<ProcessorIPCData>>
}

#[async_trait]
impl VoiceEventHandler for TrackErrorNotifier {
    async fn act(&self, ctx: &EventContext<'_>) -> Option<Event> {
        if let EventContext::Track(track_list) = ctx {
            for (state, handle) in *track_list {
                self.error_reporter.clone()(ErrorReport {
                    error: format!( "Track {:?} encountered an error: {:?}", handle.uuid(), state.playing),
                    request_id: self.request_id.clone(),
                    job_id: self.job_id.clone().to_string(),
                    guild_id: self.guild_id.clone(),
                }, &self.config)
            }
        }

        None
    }
}

#[async_trait]
impl VoiceEventHandler for TrackEndNotifier {
    async fn act(&self, _ctx: &EventContext<'_>) -> Option<Event> {
        let x = self.tx.send(ProcessorIPCData {
            action_type: ProcessorIncomingAction::Infrastructure(Infrastructure::TrackEnded),
            songbird: None,
            dwc: None,
            error_report: None,
            job_id: self.job_id.clone()
        });
        match x {
            Ok(_) => {},
            Err(_e) => {
                error!("Failed to notify job: {} that track has ended.",self.job_id.to_string());
            }
        }

        None
    }
}

pub async fn join_channel(guild_id: String, voice_channel_id: String, job_id: JobID, request_id: String, manager: &mut Option<Arc<Songbird>>, error_reporter: fn(ErrorReport, &Config), config: Config,tx: Arc<Sender<ProcessorIPCData>>) -> Result<()> {
    let gid = guild_id.parse().context(ChannelControlError::GuildIDParsingFailed.to_string())?;
    let vcid = voice_channel_id.parse().context(ChannelControlError::ChannelIDParsingFailed.to_string())?;
    if let Ok(handler_lock) = manager.as_mut().context(ChannelControlError::ManagerAcquisitionFailed.to_string())?.join(GuildId(gid), ChannelId(vcid)).await {
        // Attach an event handler to see notifications of all track errors.
        let mut handler = handler_lock.lock().await;
        handler.add_global_event(TrackEvent::Error.into(), TrackErrorNotifier {
            job_id: job_id.clone(),
            error_reporter,
            request_id,
            config,
            guild_id
        });
        // Attach notifier that tells the job that the track is completed so that it is marked thusly
        handler.add_global_event(TrackEvent::End.into(),TrackEndNotifier { job_id, tx });
    }
    Ok(())
}