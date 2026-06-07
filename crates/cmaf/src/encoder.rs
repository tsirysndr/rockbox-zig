//! Encoder loop: pull raw S16LE stereo PCM from the intake, feed it to
//! fdk-aac one 1024-sample frame at a time, accumulate
//! [`crate::FRAMES_PER_SEGMENT`] frames into a complete fMP4 segment, and
//! publish to the segment store.

use std::sync::Arc;

use fdk_aac::enc::{AudioObjectType, BitRate, ChannelMode, Encoder, EncoderParams, Transport};

use crate::{
    intake, mp4, PcmIntake, Segment, SegmentStore, AAC_FRAME_SAMPLES, CHANNELS, FRAMES_PER_SEGMENT,
    SAMPLE_RATE,
};

/// 1024 samples × 2 channels × 2 bytes/sample.
const FRAME_BYTES_PCM: usize = AAC_FRAME_SAMPLES * (CHANNELS as usize) * 2;

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

    // Init segment is fixed for our hard-coded AAC-LC / 44.1 kHz / stereo
    // config, so we can build it before the first PCM arrives.
    let init_seg = mp4::write_init_segment();
    store.set_init(init_seg);

    let mut output = vec![0u8; 8192];
    let mut frames_in_seg: Vec<Vec<u8>> = Vec::with_capacity(FRAMES_PER_SEGMENT);
    let mut next_seq: u64 = 1;
    let mut base_media_decode_time: u64 = 0;

    let intake = intake();
    let mut pending: Vec<u8> = Vec::with_capacity(FRAME_BYTES_PCM * 4);

    loop {
        // Refill `pending` until we have at least one full encoder frame
        // (or the intake is closed).
        while pending.len() < FRAME_BYTES_PCM {
            match intake.pull_at_least(FRAME_BYTES_PCM - pending.len()) {
                Some(chunk) => pending.extend_from_slice(&chunk),
                None => {
                    tracing::info!("cmaf encoder: intake closed, exiting");
                    return Ok(());
                }
            }
        }

        // Re-interpret the head of `pending` as i16 samples.
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
            // Encoder needs more priming frames — skip emit, keep feeding.
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
