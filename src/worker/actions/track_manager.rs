use std::time::Duration;


use snafu::{OptionExt, ResultExt, Whatever};
use songbird::input::Metadata;
use songbird::tracks::{Action, TrackHandle, View};
use symphonia_core::meta::Tag;

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

fn get_probed_metadata(meta: &mut Metadata) -> Result<Vec<Tag>,Whatever> {
    let probed = meta.probe.get().with_whatever_context(|| "Failed to get probed metadata")?;
    let tags = probed.current().with_whatever_context(|| "Failed to get current metadata")?.tags();
    Ok(tags.to_vec())
}

fn get_format_metadata(meta: &Metadata) -> Result<Vec<Tag>,Whatever> {
    let format = meta.format.current().with_whatever_context(|| "Failed to get format metadata")?;
    let tags = format.tags();
    Ok(tags.to_vec())
}

fn get_metadata_action(view: View) -> Option<Action> {
    let mut meta = view.meta.unwrap();
    let tags = get_probed_metadata(&mut meta);
    match tags {
        Ok(t) => {
            println!("{:?}",t)
        },
        Err(e) => {
            println!("Probed failed with: {}",e);
            // This is fine for testing
            let tags = get_format_metadata(&meta).unwrap();
            println!("{:?}",tags);
        }
    }
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