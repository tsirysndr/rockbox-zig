//! HLS manifest writers — master.m3u8 + audio media playlist.

use std::fmt::Write;

use crate::{Segment, AAC_FRAME_SAMPLES, FRAMES_PER_SEGMENT, SAMPLE_RATE};

/// Master playlist with a single audio rendition.
pub(crate) fn master_m3u8() -> String {
    let bitrate = crate::bitrate_bps();
    let mut s = String::new();
    s.push_str("#EXTM3U\n");
    s.push_str("#EXT-X-VERSION:7\n");
    s.push_str("#EXT-X-INDEPENDENT-SEGMENTS\n");
    writeln!(
        s,
        "#EXT-X-STREAM-INF:BANDWIDTH={bitrate},CODECS=\"mp4a.40.2\",AUDIO=\"audio\""
    )
    .unwrap();
    s.push_str("audio.m3u8\n");
    s
}

/// Media playlist showing the sliding window of recent segments.
pub(crate) fn audio_m3u8(media_sequence: u64, segments: &[Segment]) -> String {
    let target = segment_duration_seconds().ceil() as u32;
    let mut s = String::new();
    s.push_str("#EXTM3U\n");
    s.push_str("#EXT-X-VERSION:7\n");
    writeln!(s, "#EXT-X-TARGETDURATION:{target}").unwrap();
    writeln!(s, "#EXT-X-MEDIA-SEQUENCE:{media_sequence}").unwrap();
    s.push_str("#EXT-X-MAP:URI=\"/init.mp4\"\n");
    for seg in segments {
        let dur = (seg.duration as f64) / (SAMPLE_RATE as f64);
        writeln!(s, "#EXTINF:{dur:.3},").unwrap();
        writeln!(s, "/seg/{}.m4s", seg.seq).unwrap();
    }
    s
}

fn segment_duration_seconds() -> f64 {
    (FRAMES_PER_SEGMENT * AAC_FRAME_SAMPLES) as f64 / SAMPLE_RATE as f64
}
