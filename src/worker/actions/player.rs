
use std::sync::Arc;
use hearth_interconnect::worker_communication::DirectWorkerCommunication;
use reqwest::Client;

use snafu::{OptionExt, ResultExt, Whatever};
use songbird::{Songbird};
use songbird::tracks::TrackHandle;
use crate::worker::actions::helpers::get_manager_call;

use snafu::Snafu;
use songbird::input::{AudioStreamError, AuxMetadata, Compose, HttpRequest, Input, MetadataError, YoutubeDl};


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
    #[snafu(display("Failed to extract metadata"))]
    FailedToExtractMetadata { source: AudioStreamError },
}

pub struct PlaybackResult {
    pub track_handle: TrackHandle,
    pub metadata: AuxMetadata
}

pub async fn play_direct_link(dwc: &DirectWorkerCommunication, manager: &mut Option<Arc<Songbird>>,client: Client) -> Result<PlaybackResult,PlaybackError> {
    let handler_lock = get_manager_call(dwc.guild_id.as_ref().context(GuildIDNotFoundSnafu)?,manager).await.context(FailedToGetHandlerLockSnafu { })?;
    let mut handler = handler_lock.lock().await;
    let mut source = HttpRequest::new(client, dwc.play_audio_url.clone().context(MissingAudioURLSnafu)?);
    let metadata = source.aux_metadata().await.unwrap();
    println!("{:?}",metadata);
    let track_handle = handler.play_input(source.into());
    Ok(PlaybackResult {
        metadata,
        track_handle
    })
}

pub async fn play_from_youtube(manager: &mut Option<Arc<Songbird>>,dwc: &DirectWorkerCommunication,client: Client) -> Result<PlaybackResult,PlaybackError> {
    let handler_lock = get_manager_call(dwc.guild_id.as_ref().context(GuildIDNotFoundSnafu)?,manager).await.context(FailedToGetHandlerLockSnafu { })?;
    let mut handler = handler_lock.lock().await;
    let mut source = YoutubeDl::new(client, dwc.play_audio_url.clone().context(MissingAudioURLSnafu)?);
    let metadata = source.aux_metadata().await.context(FailedToExtractMetadataSnafu)?;
    let track_handle = handler.play_input(source.into());
    Ok(PlaybackResult {
        metadata,
        track_handle
    })
}