//! DASH MPD writer — dynamic live profile, pointing at the same fMP4 segments.

use std::fmt::Write;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::{AAC_FRAME_SAMPLES, FRAMES_PER_SEGMENT, SAMPLE_RATE, SEGMENT_WINDOW};

/// `start_unix_ms` is the wall-clock time when the encoder pushed the
/// init segment (i.e. when segment seq=1 became available).
pub(crate) fn manifest_mpd(start_unix_ms: u64, latest_seq: u64) -> String {
    let availability_start = format_unix_ms_as_iso8601(start_unix_ms);
    let publish_time = format_unix_ms_as_iso8601(now_unix_ms());

    let timescale = SAMPLE_RATE;
    let seg_duration = (FRAMES_PER_SEGMENT * AAC_FRAME_SAMPLES) as u32;
    let seg_seconds = (seg_duration as f64) / (timescale as f64);
    let time_shift = (SEGMENT_WINDOW as f64 * seg_seconds).max(seg_seconds);
    let mup = seg_seconds; // minimumUpdatePeriod
    let suggested_pres_delay = seg_seconds * 3.0;
    let bitrate = crate::bitrate_bps();

    // startNumber should match the absolute segment number that will be
    // visible at availabilityStartTime + 0. Use 1 (we always reset seqs
    // when the stream starts).
    let _ = latest_seq;

    let mut s = String::new();
    s.push_str("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
    writeln!(
        s,
        "<MPD xmlns=\"urn:mpeg:dash:schema:mpd:2011\" \
profiles=\"urn:mpeg:dash:profile:isoff-live:2011\" \
type=\"dynamic\" \
availabilityStartTime=\"{availability_start}\" \
publishTime=\"{publish_time}\" \
minimumUpdatePeriod=\"PT{mup:.3}S\" \
timeShiftBufferDepth=\"PT{time_shift:.3}S\" \
suggestedPresentationDelay=\"PT{suggested_pres_delay:.3}S\" \
minBufferTime=\"PT{seg_seconds:.3}S\">"
    )
    .unwrap();

    s.push_str("  <Period id=\"0\" start=\"PT0S\">\n");
    writeln!(
        s,
        "    <AdaptationSet id=\"0\" contentType=\"audio\" mimeType=\"audio/mp4\" \
codecs=\"mp4a.40.2\" segmentAlignment=\"true\" startWithSAP=\"1\">"
    )
    .unwrap();
    writeln!(
        s,
        "      <AudioChannelConfiguration \
schemeIdUri=\"urn:mpeg:dash:23003:3:audio_channel_configuration:2011\" value=\"2\"/>"
    )
    .unwrap();
    writeln!(
        s,
        "      <Representation id=\"audio0\" bandwidth=\"{bitrate}\" \
audioSamplingRate=\"{SAMPLE_RATE}\">"
    )
    .unwrap();
    writeln!(
        s,
        "        <SegmentTemplate timescale=\"{timescale}\" duration=\"{seg_duration}\" \
startNumber=\"1\" initialization=\"/init.mp4\" media=\"/seg/$Number$.m4s\"/>"
    )
    .unwrap();
    s.push_str("      </Representation>\n");
    s.push_str("    </AdaptationSet>\n");
    s.push_str("  </Period>\n");
    s.push_str("</MPD>\n");
    s
}

fn now_unix_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

/// Minimal ISO-8601 in UTC ("YYYY-MM-DDTHH:MM:SS.mmmZ") without bringing in
/// chrono. Uses the proleptic-Gregorian / civil-from-days algorithm
/// (Howard Hinnant's date library).
fn format_unix_ms_as_iso8601(ms: u64) -> String {
    let secs = (ms / 1000) as i64;
    let millis = (ms % 1000) as u32;

    let days = secs.div_euclid(86_400);
    let secs_in_day = secs.rem_euclid(86_400);
    let hour = (secs_in_day / 3600) as u32;
    let minute = ((secs_in_day % 3600) / 60) as u32;
    let second = (secs_in_day % 60) as u32;

    // 1970-01-01 is day 0 in this calendar; civil_from_days expects
    // shifted-epoch days where day 0 = 0000-03-01.
    let z = days + 719_468;
    let era = if z >= 0 {
        z / 146_097
    } else {
        (z - 146_096) / 146_097
    };
    let doe = (z - era * 146_097) as u64;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let y = (yoe as i64) + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let year = if m <= 2 { y + 1 } else { y };

    format!("{year:04}-{m:02}-{d:02}T{hour:02}:{minute:02}:{second:02}.{millis:03}Z")
}
