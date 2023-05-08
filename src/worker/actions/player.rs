use std::fmt::format;
use std::sync::Arc;
use serenity::model::id::GuildId;
use snafu::{OptionExt, ResultExt, Whatever, whatever};
use songbird::Songbird;
use songbird::tracks::TrackHandle;
use crate::utils::generic_connector::DirectWorkerCommunication;
use crate::worker::actions::helpers::get_manager_call;
use crate::worker::queue_processor::ErrorReport;
use crate::worker::sources::url::url_source;
use snafu::Snafu;
use crate::worker::actions::player::PlaybackError::YoutubeSourceAcquisitionFailure;

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


pub async fn play_direct_link(dwc: &DirectWorkerCommunication, mut manager: &mut Option<Arc<Songbird>>) -> Result<TrackHandle,PlaybackError> {
    let handler_lock = get_manager_call(dwc.guild_id.as_ref().context(GuildIDNotFoundSnafu)?,manager).await.context(FailedToGetHandlerLockSnafu { })?;
    let mut handler = handler_lock.lock().await;
    let source = url_source(dwc.play_audio_url.as_ref().context(MissingAudioURLSnafu)?.as_str()).await.context(DirectSourceAcquisitionFailureSnafu)?;
    // let source = songbird::ffmpeg("serverless.wav").await.unwrap();
    let track = handler.play_source(source);
    Ok(track)
}

pub async fn play_from_youtube(mut manager: &mut Option<Arc<Songbird>>,dwc: &DirectWorkerCommunication) -> Result<TrackHandle,PlaybackError> {
    let handler_lock = get_manager_call(dwc.guild_id.as_ref().context(GuildIDNotFoundSnafu)?,manager).await.context(FailedToGetHandlerLockSnafu { })?;
    let mut handler = handler_lock.lock().await;
    let source = songbird::ytdl(dwc.play_audio_url.as_ref().context(MissingAudioURLSnafu)?).await;
    match source {
        Ok(s) => {
            let track = handler.play_source(s);
            Ok(track)
        },
        Err(e) => Err(YoutubeSourceAcquisitionFailure {})
    }
}