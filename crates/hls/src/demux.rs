//! Light helper: probe an fMP4 init segment to read the codec hint up-front.
//! The actual demuxing happens in `decoder::decode_segment` where the init
//! and media segments are concatenated and parsed in one pass.

use anyhow::{anyhow, Result};
use bytes::Bytes;
use std::io::Cursor;
use symphonia::core::codecs::CODEC_TYPE_AAC;
use symphonia::core::formats::FormatOptions;
use symphonia::core::io::MediaSourceStream;
use symphonia::core::meta::MetadataOptions;
use symphonia::core::probe::Hint;

#[derive(Debug, Clone, Default)]
pub struct DecoderHints {
    pub sample_rate: Option<u32>,
    pub channels: Option<u16>,
}

pub fn parse_init(init: &[u8]) -> Result<DecoderHints> {
    let cursor = Cursor::new(Bytes::copy_from_slice(init));
    let mss = MediaSourceStream::new(Box::new(cursor), Default::default());
    let mut hint = Hint::new();
    hint.with_extension("mp4");
    let probed = symphonia::default::get_probe()
        .format(
            &hint,
            mss,
            &FormatOptions::default(),
            &MetadataOptions::default(),
        )
        .map_err(|e| anyhow!("probe init: {e}"))?;
    let track = probed
        .format
        .tracks()
        .iter()
        .find(|t| t.codec_params.codec == CODEC_TYPE_AAC)
        .ok_or_else(|| anyhow!("init segment contains no AAC track"))?;
    Ok(DecoderHints {
        sample_rate: track.codec_params.sample_rate,
        channels: track.codec_params.channels.map(|c| c.count() as u16),
    })
}
