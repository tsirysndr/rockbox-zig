//! Symphonia-based pipeline that combines demuxing and decoding.
//!
//! One call: `decode_segment(init_bytes?, seg_bytes) -> Vec<DecodedPcm>`.
//! Each `DecodedPcm` carries interleaved S16LE samples plus the sample-rate /
//! channel-count we discovered. We return a vector rather than streaming
//! because audio segments are small (~2 s) and it keeps the player loop flat.

use anyhow::{anyhow, Result};
use bytes::Bytes;
use std::io::Cursor;
use symphonia::core::audio::{AudioBufferRef, Signal};
use symphonia::core::codecs::{DecoderOptions, CODEC_TYPE_AAC};
use symphonia::core::conv::{ConvertibleSample, IntoSample};
use symphonia::core::errors::Error as SymphoniaError;
use symphonia::core::formats::FormatOptions;
use symphonia::core::io::MediaSourceStream;
use symphonia::core::meta::MetadataOptions;
use symphonia::core::probe::Hint;
use symphonia::core::sample::Sample;

pub struct DecodedSegment {
    pub samples: Vec<i16>,
    pub sample_rate: u32,
    pub channels: u16,
}

pub fn decode_segment(init: Option<&[u8]>, segment: &[u8]) -> Result<DecodedSegment> {
    // For fMP4 we must concatenate the init segment (ftyp+moov) ahead of the
    // media segment so symphonia's mp4 demuxer sees a complete file. MPEG-TS
    // segments are self-contained — pass them through as-is.
    let mut data = Vec::with_capacity(init.map_or(0, |i| i.len()) + segment.len());
    if let Some(i) = init {
        data.extend_from_slice(i);
    }
    data.extend_from_slice(segment);
    let cursor = Cursor::new(Bytes::from(data));
    let mss = MediaSourceStream::new(Box::new(cursor), Default::default());

    let probed = symphonia::default::get_probe()
        .format(
            &Hint::new(),
            mss,
            &FormatOptions::default(),
            &MetadataOptions::default(),
        )
        .map_err(|e| anyhow!("probe: {e}"))?;
    let mut format = probed.format;

    let track = format
        .tracks()
        .iter()
        .find(|t| t.codec_params.codec == CODEC_TYPE_AAC)
        .ok_or_else(|| anyhow!("no AAC track in segment"))?;
    let track_id = track.id;

    let mut decoder = symphonia::default::get_codecs()
        .make(&track.codec_params, &DecoderOptions::default())
        .map_err(|e| anyhow!("make decoder: {e}"))?;

    let mut samples: Vec<i16> = Vec::with_capacity(8192);
    let mut sample_rate = track.codec_params.sample_rate.unwrap_or(44_100);
    let mut channels = track
        .codec_params
        .channels
        .map(|c| c.count() as u16)
        .unwrap_or(2);

    loop {
        let packet = match format.next_packet() {
            Ok(p) => p,
            Err(SymphoniaError::IoError(ref e))
                if e.kind() == std::io::ErrorKind::UnexpectedEof =>
            {
                break
            }
            Err(SymphoniaError::ResetRequired) => break,
            Err(e) => return Err(anyhow!("next_packet: {e}")),
        };
        if packet.track_id() != track_id {
            continue;
        }
        let decoded = match decoder.decode(&packet) {
            Ok(d) => d,
            Err(SymphoniaError::DecodeError(msg)) => {
                tracing::warn!("hls/decoder: drop bad frame: {msg}");
                continue;
            }
            Err(e) => return Err(anyhow!("decode: {e}")),
        };
        let spec = *decoded.spec();
        sample_rate = spec.rate;
        channels = spec.channels.count() as u16;
        append_interleaved_s16(&decoded, &mut samples);
    }

    Ok(DecodedSegment {
        samples,
        sample_rate,
        channels,
    })
}

/// Symphonia's AudioBuffer is planar (one Vec per channel). Interleave it
/// into S16LE for the PCM sink, applying the standard format conversion via
/// `IntoSample`. Works for any source format symphonia emits.
fn append_interleaved_s16(decoded: &AudioBufferRef<'_>, out: &mut Vec<i16>) {
    match decoded {
        AudioBufferRef::U8(b) => copy(b.as_ref(), out),
        AudioBufferRef::U16(b) => copy(b.as_ref(), out),
        AudioBufferRef::U24(b) => copy(b.as_ref(), out),
        AudioBufferRef::U32(b) => copy(b.as_ref(), out),
        AudioBufferRef::S8(b) => copy(b.as_ref(), out),
        AudioBufferRef::S16(b) => copy(b.as_ref(), out),
        AudioBufferRef::S24(b) => copy(b.as_ref(), out),
        AudioBufferRef::S32(b) => copy(b.as_ref(), out),
        AudioBufferRef::F32(b) => copy(b.as_ref(), out),
        AudioBufferRef::F64(b) => copy(b.as_ref(), out),
    }
}

fn copy<S>(buf: &symphonia::core::audio::AudioBuffer<S>, out: &mut Vec<i16>)
where
    S: Sample + ConvertibleSample + IntoSample<i16> + Copy,
{
    let spec = buf.spec();
    let ch = spec.channels.count();
    let frames = buf.frames();
    out.reserve(frames * ch);
    for f in 0..frames {
        for c in 0..ch {
            let plane = buf.chan(c);
            out.push(plane[f].into_sample());
        }
    }
}
