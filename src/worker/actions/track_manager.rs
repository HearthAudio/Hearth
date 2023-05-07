use snafu::{OptionExt, ResultExt, Whatever};
use songbird::tracks::TrackHandle;

pub async fn pause_playback(track: &Option<TrackHandle>) -> Result<(),Whatever> {
    let t = track.as_ref().with_whatever_context(|| format!("Track not found"))?;
    let _ = t.pause().with_whatever_context(|e| format!("Failed to pause track with error: {}",e))?;
    Ok(())
}

pub async fn resume_playback(track: &Option<TrackHandle>) -> Result<(),Whatever> {
    let t = track.as_ref().with_whatever_context(|| format!("Track not found"))?;
    let _ = t.play().with_whatever_context(|e| format!("Failed to play track with error: {}",e))?;
    Ok(())
}

pub async fn set_playback_volume(track: &Option<TrackHandle>,volume: Option<f32>) -> Result<(),Whatever> {
    let t = track.as_ref().with_whatever_context(|| format!("Track not found"))?;
    let _ = t.set_volume(volume.with_whatever_context(|| "Failed to get Volume from request")?).with_whatever_context(|e| format!("Failed to set volume with error: {}",e))?;
    Ok(())
}