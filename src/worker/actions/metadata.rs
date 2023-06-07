use crate::config::Config;
use crate::worker::connector::{send_message, WORKER_PRODUCER};
use anyhow::{Context, Result};
use hearth_interconnect::errors::ErrorReport;
use hearth_interconnect::messages::{Message, Metadata};
use songbird::tracks::{Action, TrackHandle, View};
use std::sync::OnceLock;
use std::thread;
use symphonia_core::codecs::CodecParameters;
use tokio::sync::Mutex;

// This is a bit of a hack to pass data into the get metadata action
// If anyone has any better ideas please let me know
static CONFIG: OnceLock<Mutex<Option<Config>>> = OnceLock::new();
static REQUEST_ID: OnceLock<Mutex<Option<String>>> = OnceLock::new();
static JOB_ID: OnceLock<Mutex<Option<String>>> = OnceLock::new();
static GUILD_ID: OnceLock<Mutex<Option<String>>> = OnceLock::new();

#[macro_export]
macro_rules! report_metadata_error {
    ($e: ident) => {
        use $crate::errors::report_error;

        let mut cx = CONFIG.get().unwrap().lock().await;
        let c = cx.as_mut();

        let mut jx = JOB_ID.get().unwrap().lock().await;
        let j = jx.as_mut();

        let mut rx = REQUEST_ID.get().unwrap().lock().await;
        let r = rx.as_mut();

        let mut gx = GUILD_ID.get().unwrap().lock().await;
        let g = gx.as_mut();

        report_error(
            ErrorReport {
                error: format!("Failed to perform Metadata Extraction with error: {}", $e),
                request_id: r.unwrap().clone(),
                job_id: j.unwrap().clone(),
                guild_id: g.unwrap().clone(),
            },
            &*c.unwrap(),
        );
    };
}

fn get_duration(codec: &CodecParameters) -> Result<Option<u64>> {
    let time_base = codec.time_base.context("Failed to get timebase")?;
    Ok(Some(
        time_base
            .calc_time(codec.n_frames.context("Failed to get N frames")?)
            .seconds,
    ))
}

async fn get_duration_wrapper(codec: &CodecParameters) -> Option<u64> {
    let duration = get_duration(codec);
    match duration {
        Ok(d) => d,
        Err(e) => {
            report_metadata_error!(e);
            None
        }
    }
}

async fn get_codec_metadata(codec: Option<CodecParameters>, position: u64) -> Result<Metadata> {
    let codec = codec.as_ref().context("Failed to get codec")?;

    let mut jx = JOB_ID.get().unwrap().lock().await;
    let j = jx.as_mut();

    let mut gx = GUILD_ID.get().unwrap().lock().await;
    let g = gx.as_mut();

    let job_id = j
        .as_ref()
        .context("Failed to get JOB ID. While getting Metadata")?;

    let guild_id = g
        .as_ref()
        .context("Failed to get JOB ID. While getting Metadata")?;

    Ok(Metadata {
        duration: get_duration_wrapper(codec).await,
        position: Some(position),
        sample_rate: Some(codec.sample_rate.context("Failed to get Sample Rate")?),
        job_id: job_id.to_string(),
        guild_id: guild_id.to_string(),
    })
}

async fn get_metadata_sub(codec: Option<CodecParameters>, position: u64) {
    let r = get_codec_metadata(codec, position).await;
    match r {
        Ok(a) => {
            let mut px = WORKER_PRODUCER.get().unwrap().lock().await;
            let p = px.as_mut();

            let mut cx = CONFIG.get().unwrap().lock().await;
            let c = cx.as_mut();

            let config = c.unwrap();
            let topic = config.kafka.kafka_topic.clone();

            send_message(
                &Message::ExternalMetadataResult(a),
                &topic,
                &mut *p.unwrap(),
            )
            .await;
        }
        Err(e) => {
            report_metadata_error!(e);
        }
    }
}

fn get_metadata_action(view: View) -> Option<Action> {
    let codec = view.codec;
    let position = view.position.as_secs();
    thread::spawn(move || {
        futures::executor::block_on(get_metadata_sub(codec, position));
    });
    None
}

pub async fn get_metadata(
    track: &Option<TrackHandle>,
    config: &Config,
    request_id: String,
    job_id: String,
    guild_id: String,
) -> Result<()> {
    let t = track.as_ref().context("Track not found")?;

    let _ = CONFIG.set(Mutex::new(Some(config.clone())));
    let _ = JOB_ID.set(Mutex::new(Some(job_id)));
    let _ = REQUEST_ID.set(Mutex::new(Some(request_id)));
    let _ = GUILD_ID.set(Mutex::new(Some(guild_id)));

    t.action(get_metadata_action).unwrap();
    Ok(())
}
