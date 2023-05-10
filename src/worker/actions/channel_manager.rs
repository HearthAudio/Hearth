use std::num::ParseIntError;
use std::sync::Arc;
use hearth_interconnect::errors::ErrorReport;
use hearth_interconnect::worker_communication::{DirectWorkerCommunication, Job};
use log::error;
use snafu::{OptionExt, ResultExt};
use songbird::id::GuildId;
use songbird::id::ChannelId;
use songbird::{Event, EventContext, Songbird};
use songbird::events::{EventHandler as VoiceEventHandler,TrackEvent};
use snafu::Snafu;
use songbird::error::JoinError;
use serenity::async_trait;

#[derive(Debug, Snafu)]
pub enum ChannelControlError {
    #[snafu(display("Guild ID Not Found"))]
    GuildIDNotFound { },
    #[snafu(display("Guild ID Parsing Failed"))]
    GuildIDParsingFailed { source: ParseIntError },
    #[snafu(display("Channel ID Parsing Failed"))]
    ChannelIDParsingFailed { source: ParseIntError },
    #[snafu(display("Failed to acquire manager"))]
    ManagerAcquisitionFailed { },
    #[snafu(display("Failed to leave channel"))]
    ChannelLeaveFailed { source: JoinError },
    #[snafu(display("Failed to join channel"))]
    ChannelJoinFailed { source: JoinError },
}

pub async fn leave_channel(dwc: &DirectWorkerCommunication, manager: &mut Option<Arc<Songbird>>) -> Result<(),ChannelControlError> {
    manager.as_mut().context(ManagerAcquisitionFailedSnafu)?.remove(GuildId(dwc.guild_id.as_ref().context(GuildIDNotFoundSnafu)?.parse().context(GuildIDParsingFailedSnafu)?)).await.context(ChannelLeaveFailedSnafu)?;
    Ok(())
}

struct TrackErrorNotifier {
    error_reporter: fn(ErrorReport),
    request_id: String,
    job_id: String
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
                })
            }
        }

        None
    }
}

pub async fn join_channel(queue_job: &Job, request_id: String, manager: &mut Option<Arc<Songbird>>,error_reporter: fn(ErrorReport)) -> Result<(),ChannelControlError> {
    let gid = queue_job.guild_id.clone().parse().context(GuildIDParsingFailedSnafu)?;
    let vcid = queue_job.voice_channel_id.clone().parse().context(ChannelIDParsingFailedSnafu)?;
    if let Ok(handler_lock) = manager.as_mut().context(ManagerAcquisitionFailedSnafu)?.join(GuildId(gid), ChannelId(vcid)).await {
        // Attach an event handler to see notifications of all track errors.
        let mut handler = handler_lock.lock().await;
        handler.add_global_event(TrackEvent::Error.into(), TrackErrorNotifier {
            error_reporter: error_reporter,
            job_id: queue_job.job_id.clone(),
            request_id: request_id
        });
    }
    Ok(())
}