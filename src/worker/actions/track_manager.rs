use std::time::Duration;

use anyhow::{Context, Result};

use songbird::tracks::TrackHandle;

pub async fn pause_playback(track: &Option<TrackHandle>) -> Result<()> {
    let t = track.as_ref().context("Track not found")?;
    t.pause().context("Failed to pause track")?;
    Ok(())
}

pub async fn resume_playback(track: &Option<TrackHandle>) -> Result<()> {
    let t = track.as_ref().context("Track not found")?;
    t.play().context("Failed to play track")?;
    Ok(())
}

pub async fn seek_to_position(track: &Option<TrackHandle>, position: Option<u64>) -> Result<()> {
    let t = track.as_ref().context("Track not found")?;
    let duration_pos = Duration::from_millis(position.context("Failed to get seek position")?);
    let _ = t.seek(duration_pos);
    Ok(())
}

pub async fn loop_x_times(track: &Option<TrackHandle>, times: Option<usize>) -> Result<()> {
    let t = track.as_ref().context("Track not found")?;
    t.loop_for(times.context("Failed to get Loop Times")?)?;
    Ok(())
}

pub async fn loop_indefinitely(track: &Option<TrackHandle>) -> Result<()> {
    let t = track.as_ref().context("Track not found")?;
    t.enable_loop()?;
    Ok(())
}

pub async fn force_stop_loop(track: &Option<TrackHandle>) -> Result<()> {
    let t = track.as_ref().context("Track not found")?;
    t.disable_loop()?;
    Ok(())
}

pub async fn set_playback_volume(track: &Option<TrackHandle>, volume: Option<f32>) -> Result<()> {
    let t = track.as_ref().context("Track not found")?;
    t.set_volume(volume.context("Failed to get Volume from request")?)
        .context("Failed to set volume")?;
    Ok(())
}
