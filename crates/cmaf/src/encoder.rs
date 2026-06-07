//! Encoder loop: pull raw S16LE stereo PCM from the intake, feed it to
//! fdk-aac one 1024-sample frame at a time, accumulate
//! [`crate::FRAMES_PER_SEGMENT`] frames into a complete fMP4 segment, and
//! publish to the segment store.
//!
//! The encoder is **dumb**: it reads whatever the intake gives it and
//! encodes it. It never injects silence into the middle of real audio —
//! the C-side `pcm-cmaf.c::cmaf_thread` paces PCM in chunks (typically
//! 100–500 ms apart) and the encoder must NOT pad the gap, or the audio
//! comes out scrambled.
//!
//! A separate [`silence_pacer`] thread takes care of keeping the manifest
//! alive when the broadcaster goes idle. It watches the intake's
//! `last_real_push` timestamp and, after a quiet period, pushes one frame
//! of silence at a time — small enough that any real-audio resume picks
//! up within ~one frame's worth of latency.

use std::sync::Arc;
use std::time::Duration;

use fdk_aac::enc::{AudioObjectType, BitRate, ChannelMode, Encoder, EncoderParams, Transport};

use crate::{
    intake, mp4, PcmIntake, Segment, SegmentStore, AAC_FRAME_SAMPLES, CHANNELS, FRAMES_PER_SEGMENT,
    SAMPLE_RATE, SEGMENT_WINDOW,
};

/// 1024 samples × 2 channels × 2 bytes/sample.
const FRAME_BYTES_PCM: usize = AAC_FRAME_SAMPLES * (CHANNELS as usize) * 2;

/// How long the broadcaster has to be quiet before the pacer starts
/// injecting silence. Long enough that a normal chunk gap (a few hundred
/// ms) never trips it, short enough that a deliberate pause / between-track
/// gap keeps producing live segments before clients run out of buffer.
const IDLE_BEFORE_PACER: Duration = Duration::from_millis(1500);

/// How often the pacer wakes up to check whether the intake needs silence.
const PACER_TICK: Duration = Duration::from_millis(100);

pub(crate) fn run(
    _intake: Arc<PcmIntake>,
    store: Arc<SegmentStore>,
    bitrate_bps: u32,
) -> Result<(), String> {
    let encoder = Encoder::new(EncoderParams {
        bit_rate: BitRate::Cbr(bitrate_bps),
        sample_rate: SAMPLE_RATE,
        transport: Transport::Raw,
        channels: ChannelMode::Stereo,
        audio_object_type: AudioObjectType::Mpeg4LowComplexity,
    })
    .map_err(|e| format!("fdk-aac: {e:?}"))?;

    let init_seg = mp4::write_init_segment();
    store.set_init(init_seg);

    let mut output = vec![0u8; 8192];
    let mut frames_in_seg: Vec<Vec<u8>> = Vec::with_capacity(FRAMES_PER_SEGMENT);
    let mut next_seq: u64 = 1;
    let mut base_media_decode_time: u64 = 0;

    // ----- Bootstrap: pre-fill the segment window with silence -----
    // hls.js / dash.js refuse to start if the very first manifest they see
    // has zero segments. Produce `SEGMENT_WINDOW` silence segments before
    // entering the main loop so clients connecting to a freshly-booted
    // rockboxd hit a healthy stream.
    let silence_samples = vec![0i16; AAC_FRAME_SAMPLES * (CHANNELS as usize)];
    for _ in 0..SEGMENT_WINDOW {
        emit_silence_segment(
            &encoder,
            &mut output,
            &silence_samples,
            &store,
            &mut next_seq,
            &mut base_media_decode_time,
        )?;
    }

    let intake = intake();

    // Spawn the silence pacer. It runs for the lifetime of the daemon and
    // only injects when the producer has been quiet AND the intake is
    // empty, so it never mixes silence into a real-audio chunk.
    {
        let pacer_intake = intake.clone();
        std::thread::Builder::new()
            .name("cmaf-silence-pacer".into())
            .spawn(move || silence_pacer(pacer_intake))
            .map_err(|e| format!("spawn pacer: {e}"))?;
    }

    let mut pending: Vec<u8> = Vec::with_capacity(FRAME_BYTES_PCM * 4);

    loop {
        // Plain blocking pull. The pacer handles "broadcaster is idle" by
        // injecting silence into the intake, so the encoder doesn't need
        // any time-based fallback — it just reads what's there.
        while pending.len() < FRAME_BYTES_PCM {
            let need = FRAME_BYTES_PCM - pending.len();
            match intake.pull_at_least(need) {
                Some(chunk) => pending.extend_from_slice(&chunk),
                None => {
                    tracing::info!("cmaf encoder: intake closed, exiting");
                    return Ok(());
                }
            }
        }

        let frame_bytes = &pending[..FRAME_BYTES_PCM];
        let mut samples_i16 = vec![0i16; AAC_FRAME_SAMPLES * CHANNELS as usize];
        for (i, chunk) in frame_bytes.chunks_exact(2).enumerate() {
            samples_i16[i] = i16::from_le_bytes([chunk[0], chunk[1]]);
        }
        pending.drain(..FRAME_BYTES_PCM);

        let info = encoder
            .encode(&samples_i16, &mut output)
            .map_err(|e| format!("fdk-aac encode: {e:?}"))?;

        if info.output_size == 0 {
            continue;
        }

        frames_in_seg.push(output[..info.output_size].to_vec());

        if frames_in_seg.len() == FRAMES_PER_SEGMENT {
            let duration_samples = (FRAMES_PER_SEGMENT * AAC_FRAME_SAMPLES) as u32;
            let bytes = mp4::write_media_segment(
                next_seq as u32,
                base_media_decode_time,
                AAC_FRAME_SAMPLES as u32,
                &frames_in_seg,
            );

            tracing::debug!(
                "cmaf: emit seg {} ({} bytes, {} frames, base_dts {})",
                next_seq,
                bytes.len(),
                FRAMES_PER_SEGMENT,
                base_media_decode_time
            );

            store.push(Segment {
                seq: next_seq,
                duration: duration_samples,
                bytes: Arc::new(bytes),
            });

            base_media_decode_time += duration_samples as u64;
            next_seq += 1;
            frames_in_seg.clear();
        }
    }
}

/// Encode `FRAMES_PER_SEGMENT` silence frames synchronously and push the
/// resulting fMP4 segment to the store. Used for the startup bootstrap
/// only — the steady-state pacer goes through the intake instead.
fn emit_silence_segment(
    encoder: &Encoder,
    output: &mut [u8],
    silence_samples: &[i16],
    store: &Arc<SegmentStore>,
    next_seq: &mut u64,
    base_media_decode_time: &mut u64,
) -> Result<(), String> {
    let mut frames: Vec<Vec<u8>> = Vec::with_capacity(FRAMES_PER_SEGMENT);
    while frames.len() < FRAMES_PER_SEGMENT {
        let info = encoder
            .encode(silence_samples, output)
            .map_err(|e| format!("fdk-aac silence encode: {e:?}"))?;
        if info.output_size > 0 {
            frames.push(output[..info.output_size].to_vec());
        }
    }
    let duration_samples = (FRAMES_PER_SEGMENT * AAC_FRAME_SAMPLES) as u32;
    let bytes = mp4::write_media_segment(
        *next_seq as u32,
        *base_media_decode_time,
        AAC_FRAME_SAMPLES as u32,
        &frames,
    );
    store.push(Segment {
        seq: *next_seq,
        duration: duration_samples,
        bytes: Arc::new(bytes),
    });
    *base_media_decode_time += duration_samples as u64;
    *next_seq += 1;
    Ok(())
}

/// Silence pacer — runs forever, injects one frame of silence at a time
/// when the broadcaster has been idle and the intake has drained. Keeps
/// the encoder fed (so segments keep being produced) without ever
/// scribbling on a real-audio chunk.
fn silence_pacer(intake: Arc<PcmIntake>) {
    // One AAC frame's worth — 23.2 ms at 44.1 kHz. Small enough that when a
    // real track resumes, at most ~23 ms of silence is queued ahead of the
    // first real PCM bytes; below the perceptual threshold.
    let one_frame_silence = vec![0u8; FRAME_BYTES_PCM];
    loop {
        std::thread::sleep(PACER_TICK);
        let (idle, buf_len) = intake.idle_and_buf_len();
        if idle < IDLE_BEFORE_PACER {
            continue; // broadcaster recently pushed real PCM — stay out
        }
        if buf_len >= FRAME_BYTES_PCM {
            continue; // encoder still has real-audio frames queued
        }
        intake.push_silence(&one_frame_silence);
    }
}
