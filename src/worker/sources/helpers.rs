use lofty::iff::wav;
use lofty::iff::wav::WavFormat;
use songbird::input::Codec;

pub fn lofty_wac_codec_to_songbird_codec(lofty_codec: &WavFormat) -> Codec {
    match lofty_codec {
        wav::WavFormat::PCM => Codec::Pcm,
        wav::WavFormat::IEEE_FLOAT => Codec::FloatPcm,
        _ => Codec::Pcm
    }
}