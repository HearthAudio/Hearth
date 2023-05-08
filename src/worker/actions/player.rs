use std::fmt::format;
use std::sync::Arc;
use reqwest::Client;
use serenity::model::id::GuildId;
use snafu::{OptionExt, ResultExt, Whatever, whatever};
use songbird::{input, Songbird};
use songbird::tracks::TrackHandle;
use crate::utils::generic_connector::DirectWorkerCommunication;
use crate::worker::actions::helpers::get_manager_call;
use crate::worker::queue_processor::ErrorReport;
use snafu::Snafu;
use songbird::input::{Compose, HttpRequest, YoutubeDl};
use crate::worker::actions::player::PlaybackError::{MissingAudioURL, YoutubeSourceAcquisitionFailure};

#[derive(Debug, Snafu)]
pub enum PlaybackError {
    #[snafu(display("Guild ID Not Found"))]
    GuildIDNotFound { },
    #[snafu(display("Failed to acquire DirectLink source"))]
    DirectSourceAcquisitionFailure { source: Whatever },
    #[snafu(display("Failed to acquire YouTube source"))]
    YoutubeSourceAcquisitionFailure { },
    #[snafu(display("Failed to get handler lock with Error: {source}"))]
    FailedToGetHandlerLock { source: Whatever },
    #[snafu(display("Missing Audio URL"))]
    MissingAudioURL { },
}


pub async fn play_direct_link(dwc: &DirectWorkerCommunication, mut manager: &mut Option<Arc<Songbird>>,client: Client) -> Result<TrackHandle,PlaybackError> {
    let handler_lock = get_manager_call(dwc.guild_id.as_ref().context(GuildIDNotFoundSnafu)?,manager).await.context(FailedToGetHandlerLockSnafu { })?;
    let mut handler = handler_lock.lock().await;
    let source = HttpRequest::new(client,dwc.play_audio_url.clone().context(MissingAudioURLSnafu)?);
    let track = handler.play_input(source.into());
    Ok(track)
}

pub async fn play_from_youtube(mut manager: &mut Option<Arc<Songbird>>,dwc: &DirectWorkerCommunication,client: Client) -> Result<TrackHandle,PlaybackError> {
    let handler_lock = get_manager_call(dwc.guild_id.as_ref().context(GuildIDNotFoundSnafu)?,manager).await.context(FailedToGetHandlerLockSnafu { })?;
    let mut handler = handler_lock.lock().await;
    let source = YoutubeDl::new(client,dwc.play_audio_url.clone().context(MissingAudioURLSnafu)?);
    let track = handler.play_input(source.into());
    Ok(track)
}