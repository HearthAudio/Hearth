use std::time::Duration;
use hearth_interconnect::messages::MetadataResult;

use snafu::{OptionExt, ResultExt, Whatever};
use songbird::tracks::{Action, TrackHandle, View};

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

fn get_metadata_action(view: View) -> Option<Action> {
    //TODO: MATCH
    let meta = view.meta.unwrap().format.current().unwrap();
    let metadata = meta.tags();
    println!("{:?}",metadata);
    // let result = MetadataResult {
    //     duration_milliseconds: 0,
    //     file_type: "".to_string(),
    //     artist: "".to_string(),
    //     name: "".to_string(),
    //     sample_rate: 0,
    // };
    None
}

pub async fn get_metadata(track: &Option<TrackHandle>) -> Result<(),Whatever> {
    let t = track.as_ref().with_whatever_context(|| format!("Track not found"))?;
    t.action(get_metadata_action).unwrap();
    Ok(())
}

pub async fn seek_to_position(track: &Option<TrackHandle>,position: Option<u64>) -> Result<(),Whatever> {
    let t = track.as_ref().with_whatever_context(|| format!("Track not found"))?;
    let duration_pos = Duration::from_millis(position.with_whatever_context(|| "Failed to get seek position")?);
    let _ = t.seek(duration_pos);
    Ok(())
}

pub async fn loop_x_times(track: &Option<TrackHandle>,times: Option<usize>) -> Result<(),Whatever> {
    let t = track.as_ref().with_whatever_context(|| format!("Track not found"))?;
    let _ = t.loop_for(times.with_whatever_context(|| "Failed to get Loop Times")?);
    Ok(())
}

pub async fn loop_indefinitely(track: &Option<TrackHandle>) -> Result<(),Whatever> {
    let t = track.as_ref().with_whatever_context(|| format!("Track not found"))?;
    let _ = t.enable_loop();
    Ok(())
}

pub async fn force_stop_loop(track: &Option<TrackHandle>) -> Result<(),Whatever> {
    let t = track.as_ref().with_whatever_context(|| format!("Track not found"))?;
    let _ = t.disable_loop();
    Ok(())
}

pub async fn set_playback_volume(track: &Option<TrackHandle>,volume: Option<f32>) -> Result<(),Whatever> {
    let t = track.as_ref().with_whatever_context(|| format!("Track not found"))?;
    let _ = t.set_volume(volume.with_whatever_context(|| "Failed to get Volume from request")?).with_whatever_context(|e| format!("Failed to set volume with error: {}",e))?;
    Ok(())
}