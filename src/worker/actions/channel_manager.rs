use std::num::ParseIntError;
use std::sync::Arc;
use snafu::{OptionExt, ResultExt};
use songbird::id::GuildId;
use songbird::id::ChannelId;
use songbird::Songbird;
use crate::scheduler::distributor::Job;
use crate::utils::generic_connector::DirectWorkerCommunication;
use snafu::Snafu;
use songbird::error::JoinError;

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

pub async fn join_channel(queue_job: &Job, manager: &mut Option<Arc<Songbird>>) -> Result<(),ChannelControlError> {
    manager.as_mut().context(ManagerAcquisitionFailedSnafu)?.join(GuildId(queue_job.guild_id.clone().parse().context(GuildIDParsingFailedSnafu)?), ChannelId(queue_job.voice_channel_id.clone().parse().context(ChannelIDParsingFailedSnafu)?)).await.context(ChannelJoinFailedSnafu)?;
    Ok(())
}