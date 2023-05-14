use snafu::{OptionExt, Whatever};
use songbird::input::Metadata;
use songbird::tracks::{Action, TrackHandle, View};
use symphonia_core::meta::Tag;

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