use std::collections::HashMap;
use std::io::{Cursor, Read};
use std::time::Duration;
use bytes::Buf;
use enum_map::enum_map;
use lofty::{AudioFile, ParseOptions};
use lofty::iff::wav;
use lofty::iff::wav::WavFormat;
use songbird::input::{Codec, Container, Input, Metadata, Reader};
use symphonia_core::audio::Channels;
use symphonia_core::formats::FormatOptions;
use symphonia_core::io::{MediaSource, MediaSourceStream, ReadOnlySource};
use symphonia_core::meta::{MetadataOptions, Tag};
use symphonia_core::probe::Hint;
use crate::worker::sources::helpers::lofty_wac_codec_to_songbird_codec;

/// Basic URL Player that downloads files from URLs into memory and plays them
/// TODO: Optimize by only loading chunks into memory at a time by chunking downloads
/// TODO: This may require some lower level work inside of Songbird/Finch
pub async fn create_url_input(url: &str) -> Input {

    let resp = reqwest::get(url).await.unwrap();
    let mut pre : Vec<u8> = vec![];

    let bytes = resp.bytes().await.unwrap().clone();
    let metadata_bytes = bytes.clone(); // This is required because for some reason read_to_end breaks the pre-buf symph

    let r = bytes.reader();

    metadata_bytes.reader().read_to_end(&mut pre).unwrap();
    let mut mock_file : Cursor<Vec<u8>> = Cursor::new(pre);

    let parsing_options = ParseOptions::new();
    let tagged_file = wav::WavFile::read_from(&mut mock_file, parsing_options).unwrap();
    let properties = tagged_file.properties();

    let src = ReadOnlySource::new(r); //TODO: Figure out how to allow seeking when using Cursor<> mock file playback does not occur even if mutated by using read_to_end playback does not occur

    let x =  Input {
        //TODO: Proper Metadata parsing so this is less shit and is not super unreliable
        metadata: Box::new(Metadata {
            track: None,
            artist: None,
            date: None,
            channels: Some(properties.channels()),
            channel: None,
            start_time: None,
            duration: Some(properties.duration()),
            sample_rate: Some(properties.sample_rate()),
            source_url: None,
            title: None,
            thumbnail: None,
        }),
        stereo: properties.channels() >= 2,
        reader: Reader::Extension(Box::new(src)),
        kind: lofty_wac_codec_to_songbird_codec(tagged_file.properties().format()),
        container: Container::Raw,
        pos: 0,
    };
    return x;
}