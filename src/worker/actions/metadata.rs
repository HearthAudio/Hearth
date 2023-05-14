use hearth_interconnect::messages::{Message, Metadata};
use lazy_static::lazy_static;
use snafu::{OptionExt, Whatever};
use songbird::tracks::{Action, TrackHandle, View};
use symphonia_core::codecs::CodecParameters;


use crate::utils::generic_connector::PRODUCER;
use crate::worker::connector::send_message;
use crate::config::Config;
use std::sync::Mutex;
use hearth_interconnect::errors::ErrorReport;

// This is a bit of a hack to pass data into the get metadata action
lazy_static! {
    static ref CONFIG: Mutex<Option<Config>> = Mutex::new(None);
    static ref REQUEST_ID: Mutex<Option<String>> = Mutex::new(None);
    static ref JOB_ID: Mutex<Option<String>> = Mutex::new(None);
}

#[macro_export]
macro_rules! report_metadata_error {
    ($e: ident) => {
        use crate::errors::report_error;

        let mut cx = CONFIG.lock().unwrap();
        let c = cx.as_mut();

        let mut jx = JOB_ID.lock().unwrap();
        let j = jx.as_mut();

        let mut rx = REQUEST_ID.lock().unwrap();
        let r = rx.as_mut();

        report_error(ErrorReport {
            error: format!("Failed to perform Metadata Extraction with error: {}",$e),
            request_id: r.unwrap().clone(),
            job_id: j.unwrap().clone(),
        }, & *c.unwrap());
    };
}

fn get_duration(codec: &CodecParameters) -> Result<Option<u64>,Whatever> {
    let time_base = codec.time_base.with_whatever_context(|| "Failed to get timebase")?;
    Ok(Some(time_base.calc_time(codec.n_frames.with_whatever_context(|| "Failed to get N frames")?).seconds))
}

fn get_duration_wrapper(codec: &CodecParameters) -> Option<u64> {
    let duration = get_duration(codec);
    match duration {
        Ok(d) => {
            d
        },
        Err(e) => {
            report_metadata_error!(e);
            None
        }
    }
}

fn get_codec_metadata(view: &View) -> Result<Metadata,Whatever> {
    let codec = view.codec.as_ref().with_whatever_context(|| "Failed to get codec")?;
    Ok(Metadata {
        duration: get_duration_wrapper(codec),
        position: Some(view.position.as_secs()),
        sample_rate: Some(codec.sample_rate.with_whatever_context(|| "Failed to get Sample Rate")?),
    })
}

fn get_metadata_action(view: View) -> Option<Action> {
    let r = get_codec_metadata(&view);
    //TODO: DO work on another thread to not slow track playback
    match r {
        Ok(a) => {
            let mut px = PRODUCER.lock().unwrap();
            let p = px.as_mut();

            send_message(&Message::ExternalMetadataResult(a),"communication",&mut *p.unwrap())
        },
        Err(e) => {
            report_metadata_error!(e);
        }
    }
    None
}

pub async fn get_metadata(track: &Option<TrackHandle>,config: &Config,request_id: String,job_id: String) -> Result<(),Whatever> {
    let t = track.as_ref().with_whatever_context(|| "Track not found")?;

    *CONFIG.lock().unwrap() = Some(config.clone());
    *JOB_ID.lock().unwrap() = Some(job_id.clone());
    *REQUEST_ID.lock().unwrap() = Some(request_id.clone());

    t.action(get_metadata_action).unwrap();
    Ok(())
}