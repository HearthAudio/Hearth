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

pub enum ChannelControlError {
    GuildIDNotFound,
    GuildIDParsingFailed,
    ChannelIDParsingFailed,
    ManagerAcquisitionFailed,
    ChannelLeaveFailed,
}

impl fmt::Display for ChannelControlError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ChannelControlError::GuildIDNotFound => write!(f, "Guild ID Not Found"),
            ChannelControlError::GuildIDParsingFailed => write!(f, "Guild ID Parsing Failed"),
            ChannelControlError::ChannelIDParsingFailed =>  write!(f, "Channel ID Parsing Failed"),
            ChannelControlError::ManagerAcquisitionFailed =>  write!(f, "Failed to acquire manager"),
            ChannelControlError::ChannelLeaveFailed => write!(f, "Failed to leave channel"),
        }
    }
}

pub async fn leave_channel(dwc: &DirectWorkerCommunication, manager: &mut Option<Arc<Songbird>>) -> Result<()> {
    manager.as_mut().context(ChannelControlError::ManagerAcquisitionFailed.to_string())?.remove(GuildId(dwc.guild_id.as_ref().context(ChannelControlError::GuildIDNotFound.to_string())?.parse().context(ChannelControlError::GuildIDParsingFailed.to_string())?)).await.context(ChannelControlError::ChannelLeaveFailed.to_string())?;
    Ok(())
}

struct TrackErrorNotifier {
    error_reporter: fn(ErrorReport,&Config),
    request_id: String,
    guild_id: String,
    job_id: String,
    config: Config
}

#[async_trait]
impl VoiceEventHandler for TrackErrorNotifier {
    async fn act(&self, ctx: &EventContext<'_>) -> Option<Event> {
        if let EventContext::Track(track_list) = ctx {
            for (state, handle) in *track_list {
                self.error_reporter.clone()(ErrorReport {
                    error: format!( "Track {:?} encountered an error: {:?}", handle.uuid(), state.playing),
                    request_id: self.request_id.clone(),
                    job_id: self.job_id.clone(),
                    guild_id: self.guild_id.clone(),
                }, &self.config)
            }
        }

        None
    }
}

pub async fn join_channel(guild_id: String, voice_channel_id: String, job_id: String, request_id: String, manager: &mut Option<Arc<Songbird>>, error_reporter: fn(ErrorReport, &Config), config: Config) -> Result<()> {
    let gid = guild_id.parse().context(ChannelControlError::GuildIDParsingFailed.to_string())?;
    let vcid = voice_channel_id.parse().context(ChannelControlError::ChannelIDParsingFailed.to_string())?;
    if let Ok(handler_lock) = manager.as_mut().context(ChannelControlError::ManagerAcquisitionFailed.to_string())?.join(GuildId(gid), ChannelId(vcid)).await {
        // Attach an event handler to see notifications of all track errors.
        let mut handler = handler_lock.lock().await;
        handler.add_global_event(TrackEvent::Error.into(), TrackErrorNotifier {
            error_reporter: error_reporter,
            job_id: job_id,
            request_id: request_id,
            config: config,
            guild_id: guild_id
        });
    }
    Ok(())
}