use std::ffi::OsStr;
use std::path::Path;
use lofty::iff::wav;
use lofty::iff::wav::WavFormat;
use songbird::input::Codec;
use url::Url;

pub fn lofty_wav_codec_to_songbird_codec(lofty_codec: &WavFormat) -> Codec {
    match lofty_codec {
        wav::WavFormat::PCM => Codec::Pcm,
        wav::WavFormat::IEEE_FLOAT => Codec::FloatPcm,
        _ => Codec::Pcm
    }
}

pub fn get_extension_from_uri(uri: &str) -> String {
    let uri = Url::parse(uri).unwrap();
    let parsed = uri.path().split(".").collect::<Vec<&str>>();
    return parsed[parsed.len() - 1].to_string();
}