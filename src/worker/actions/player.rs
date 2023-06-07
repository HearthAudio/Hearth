use crate::worker::actions::helpers::get_manager_call;
use anyhow::{Context, Result};
use hearth_interconnect::worker_communication::DirectWorkerCommunication;
use reqwest::Client;
use songbird::input::{HttpRequest, YoutubeDl};
use songbird::tracks::TrackHandle;
use songbird::Songbird;
use std::fmt;
use std::sync::Arc;

#[derive(Debug)]
enum PlaybackError {
    MissingAudioURL,
}

impl fmt::Display for PlaybackError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            PlaybackError::MissingAudioURL => write!(f, "Missing Audio URL"),
        }
    }
}

pub async fn play_direct_link(
    dwc: &DirectWorkerCommunication,
    manager: &mut Option<Arc<Songbird>>,
    client: Client,
) -> Result<TrackHandle> {
    let handler_lock = get_manager_call(&dwc.guild_id, manager).await?;
    let mut handler = handler_lock.lock().await;
    let source = HttpRequest::new(
        client,
        dwc.play_audio_url
            .clone()
            .context(PlaybackError::MissingAudioURL.to_string())?,
    );
    let track_handle = handler.play_input(source.into());
    Ok(track_handle)
}

pub async fn play_from_youtube(
    manager: &mut Option<Arc<Songbird>>,
    dwc: &DirectWorkerCommunication,
    client: Client,
) -> Result<TrackHandle> {
    let handler_lock = get_manager_call(&dwc.guild_id, manager).await?;
    let mut handler = handler_lock.lock().await;
    let source = YoutubeDl::new(
        client,
        dwc.play_audio_url
            .clone()
            .context(PlaybackError::MissingAudioURL.to_string())?,
    );
    let track_handle = handler.play_input(source.into());
    Ok(track_handle)
}
