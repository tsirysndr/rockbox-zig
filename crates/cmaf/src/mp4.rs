//! Hand-rolled fragmented-MP4 (CMAF audio profile) writer.
//!
//! Produces a single init segment (ftyp + moov) and any number of media
//! segments (styp + moof + mdat). All boxes are written big-endian per
//! ISO/IEC 14496-12. Only what is needed for AAC-LC stereo @ 44.1 kHz is
//! implemented; sample rate / channels are baked in via constants.

use crate::{CHANNELS, SAMPLE_RATE};

const TRACK_ID: u32 = 1;

// ---------------------------------------------------------------------------
// Box-writing primitives
// ---------------------------------------------------------------------------

struct Writer {
    buf: Vec<u8>,
}

impl Writer {
    fn new() -> Self {
        Writer {
            buf: Vec::with_capacity(1024),
        }
    }

    fn into_vec(self) -> Vec<u8> {
        self.buf
    }

    fn u8(&mut self, v: u8) {
        self.buf.push(v);
    }
    fn u16(&mut self, v: u16) {
        self.buf.extend_from_slice(&v.to_be_bytes());
    }
    fn u24(&mut self, v: u32) {
        let b = v.to_be_bytes();
        self.buf.extend_from_slice(&b[1..4]);
    }
    fn u32(&mut self, v: u32) {
        self.buf.extend_from_slice(&v.to_be_bytes());
    }
    fn u64(&mut self, v: u64) {
        self.buf.extend_from_slice(&v.to_be_bytes());
    }
    fn bytes(&mut self, v: &[u8]) {
        self.buf.extend_from_slice(v);
    }
    fn fourcc(&mut self, v: &[u8; 4]) {
        self.buf.extend_from_slice(v);
    }

    fn len(&self) -> usize {
        self.buf.len()
    }

    /// Begin a box and return its start offset (for back-patching the size).
    fn begin_box(&mut self, kind: &[u8; 4]) -> usize {
        let start = self.buf.len();
        self.u32(0); // placeholder size
        self.fourcc(kind);
        start
    }

    /// Begin a full box (with version+flags) and return its start offset.
    fn begin_full_box(&mut self, kind: &[u8; 4], version: u8, flags: u32) -> usize {
        let start = self.begin_box(kind);
        self.u8(version);
        self.u24(flags & 0x00FF_FFFF);
        start
    }

    fn end_box(&mut self, start: usize) {
        let size = (self.buf.len() - start) as u32;
        self.buf[start..start + 4].copy_from_slice(&size.to_be_bytes());
    }
}

// ---------------------------------------------------------------------------
// AudioSpecificConfig for AAC-LC 44.1 kHz stereo:
//
//   audioObjectType        = 2 (AAC-LC)        — 5 bits  = 00010
//   samplingFrequencyIndex = 4 (44100 Hz)      — 4 bits  = 0100
//   channelConfiguration   = 2 (stereo)        — 4 bits  = 0010
//   GASpecificConfig       = 0,0,0             — 3 bits  = 000
//
// Concatenated: 00010 0100 0010 000  →  0001 0010 0001 0000  =  0x12 0x10
// ---------------------------------------------------------------------------
const AUDIO_SPECIFIC_CONFIG: [u8; 2] = [0x12, 0x10];

// ---------------------------------------------------------------------------
// Init segment (ftyp + moov)
// ---------------------------------------------------------------------------

pub(crate) fn write_init_segment() -> Vec<u8> {
    let mut w = Writer::new();
    write_ftyp(&mut w);
    write_moov(&mut w);
    w.into_vec()
}

fn write_ftyp(w: &mut Writer) {
    let s = w.begin_box(b"ftyp");
    w.fourcc(b"iso5"); // major brand
    w.u32(0x0000_0200); // minor version
    w.fourcc(b"iso6");
    w.fourcc(b"mp41");
    w.fourcc(b"cmfc");
    w.end_box(s);
}

fn write_moov(w: &mut Writer) {
    let s = w.begin_box(b"moov");
    write_mvhd(w);
    write_trak(w);
    write_mvex(w);
    w.end_box(s);
}

fn write_mvhd(w: &mut Writer) {
    // version 0
    let s = w.begin_full_box(b"mvhd", 0, 0);
    w.u32(0); // creation_time
    w.u32(0); // modification_time
    w.u32(SAMPLE_RATE); // timescale
    w.u32(0); // duration = 0 (fragmented)
    w.u32(0x0001_0000); // rate = 1.0
    w.u16(0x0100); // volume = 1.0
    w.u16(0); // reserved
    w.u32(0);
    w.u32(0); // reserved (8 bytes)
    write_unity_matrix(w);
    for _ in 0..6 {
        w.u32(0);
    } // pre_defined
    w.u32(TRACK_ID + 1); // next_track_id
    w.end_box(s);
}

fn write_unity_matrix(w: &mut Writer) {
    // 1, 0, 0, 0, 1, 0, 0, 0, 1 in 16.16 / 2.30 fixed point
    let m: [u32; 9] = [0x0001_0000, 0, 0, 0, 0x0001_0000, 0, 0, 0, 0x4000_0000];
    for v in m {
        w.u32(v);
    }
}

fn write_trak(w: &mut Writer) {
    let s = w.begin_box(b"trak");
    write_tkhd(w);
    write_mdia(w);
    w.end_box(s);
}

fn write_tkhd(w: &mut Writer) {
    // version 0, flags = 0x000007 (track enabled, in movie, in preview)
    let s = w.begin_full_box(b"tkhd", 0, 0x07);
    w.u32(0); // creation_time
    w.u32(0); // modification_time
    w.u32(TRACK_ID);
    w.u32(0); // reserved
    w.u32(0); // duration
    w.u32(0);
    w.u32(0); // reserved (8 bytes)
    w.u16(0); // layer
    w.u16(0); // alternate_group
    w.u16(0x0100); // volume = 1.0 (audio)
    w.u16(0); // reserved
    write_unity_matrix(w);
    w.u32(0); // width  (audio)
    w.u32(0); // height (audio)
    w.end_box(s);
}

fn write_mdia(w: &mut Writer) {
    let s = w.begin_box(b"mdia");
    write_mdhd(w);
    write_hdlr(w);
    write_minf(w);
    w.end_box(s);
}

fn write_mdhd(w: &mut Writer) {
    let s = w.begin_full_box(b"mdhd", 0, 0);
    w.u32(0); // creation_time
    w.u32(0); // modification_time
    w.u32(SAMPLE_RATE);
    w.u32(0); // duration
              // language = 'und' packed (0x55C4)
    w.u16(0x55C4);
    w.u16(0); // pre_defined
    w.end_box(s);
}

fn write_hdlr(w: &mut Writer) {
    let s = w.begin_full_box(b"hdlr", 0, 0);
    w.u32(0); // pre_defined
    w.fourcc(b"soun"); // handler_type
    w.u32(0);
    w.u32(0);
    w.u32(0); // reserved (12)
    w.bytes(b"SoundHandler\0");
    w.end_box(s);
}

fn write_minf(w: &mut Writer) {
    let s = w.begin_box(b"minf");
    write_smhd(w);
    write_dinf(w);
    write_stbl(w);
    w.end_box(s);
}

fn write_smhd(w: &mut Writer) {
    let s = w.begin_full_box(b"smhd", 0, 0);
    w.u16(0); // balance
    w.u16(0); // reserved
    w.end_box(s);
}

fn write_dinf(w: &mut Writer) {
    let s = w.begin_box(b"dinf");
    let dref = w.begin_full_box(b"dref", 0, 0);
    w.u32(1); // entry_count
              // url with flags=1 (self-contained, no location string)
    let url = w.begin_full_box(b"url ", 0, 0x01);
    w.end_box(url);
    w.end_box(dref);
    w.end_box(s);
}

fn write_stbl(w: &mut Writer) {
    let s = w.begin_box(b"stbl");
    write_stsd(w);
    // Empty sample tables — all real samples live in moof/trun.
    let stts = w.begin_full_box(b"stts", 0, 0);
    w.u32(0);
    w.end_box(stts);
    let stsc = w.begin_full_box(b"stsc", 0, 0);
    w.u32(0);
    w.end_box(stsc);
    let stsz = w.begin_full_box(b"stsz", 0, 0);
    w.u32(0); // sample_size
    w.u32(0); // sample_count
    w.end_box(stsz);
    let stco = w.begin_full_box(b"stco", 0, 0);
    w.u32(0);
    w.end_box(stco);
    w.end_box(s);
}

fn write_stsd(w: &mut Writer) {
    let s = w.begin_full_box(b"stsd", 0, 0);
    w.u32(1); // entry_count
    write_mp4a(w);
    w.end_box(s);
}

fn write_mp4a(w: &mut Writer) {
    // AudioSampleEntry / mp4a
    let s = w.begin_box(b"mp4a");
    // SampleEntry header
    for _ in 0..6 {
        w.u8(0);
    } // reserved
    w.u16(1); // data_reference_index
              // AudioSampleEntry
    w.u32(0);
    w.u32(0); // reserved (8 bytes)
    w.u16(CHANNELS); // channelcount
    w.u16(16); // samplesize
    w.u16(0); // pre_defined
    w.u16(0); // reserved
    w.u32(SAMPLE_RATE << 16); // samplerate (16.16 fixed)
    write_esds(w);
    w.end_box(s);
}

fn write_esds(w: &mut Writer) {
    let s = w.begin_full_box(b"esds", 0, 0);

    // ES_Descriptor (tag 0x03). We compute its inner-payload size first.
    // Payload = 3 (ES_ID + flags) + DecoderConfigDescriptor + SLConfigDescriptor
    //
    //   DecoderConfigDescriptor (tag 4):
    //     1 (oti) + 1 (streamType packed) + 3 (bufferSizeDB) + 4 (maxBitrate)
    //     + 4 (avgBitrate)
    //     + DecoderSpecificInfo (tag 5, contains AudioSpecificConfig)
    //
    //   SLConfigDescriptor (tag 6, predefined = 0x02): 1 byte payload
    //
    // Sizes are encoded as one byte each (always < 128 for our use).
    let asc_len = AUDIO_SPECIFIC_CONFIG.len() as u8;
    let dsi_payload = asc_len; // payload only (excludes tag+size byte)
    let dsi_total = 2 + dsi_payload; // tag(1) + size(1) + payload

    let dcd_payload = 1 + 1 + 3 + 4 + 4 + dsi_total;
    let dcd_total = 2 + dcd_payload;

    let sl_total = 2 + 1; // tag + size + 1 payload byte

    let esd_payload = 3 + dcd_total + sl_total;

    // ES_Descriptor
    w.u8(0x03);
    w.u8(esd_payload);
    w.u16(0x0001); // ES_ID
    w.u8(0x00); // flags (no streamDependence/url/ocr, streamPriority=0)

    // DecoderConfigDescriptor
    w.u8(0x04);
    w.u8(dcd_payload);
    w.u8(0x40); // objectTypeIndication = MPEG-4 Audio
                // streamType=5 (Audio), upStream=0, reserved=1  →  (5<<2) | 1 = 0x15
    w.u8(0x15);
    w.u24(0); // bufferSizeDB
    w.u32(crate::bitrate_bps()); // maxBitrate
    w.u32(crate::bitrate_bps()); // avgBitrate

    // DecoderSpecificInfo
    w.u8(0x05);
    w.u8(dsi_payload);
    w.bytes(&AUDIO_SPECIFIC_CONFIG);

    // SLConfigDescriptor
    w.u8(0x06);
    w.u8(0x01);
    w.u8(0x02); // predefined = MP4

    w.end_box(s);
}

fn write_mvex(w: &mut Writer) {
    let s = w.begin_box(b"mvex");
    // trex
    let trex = w.begin_full_box(b"trex", 0, 0);
    w.u32(TRACK_ID);
    w.u32(1); // default_sample_description_index
    w.u32(0); // default_sample_duration
    w.u32(0); // default_sample_size
    w.u32(0); // default_sample_flags
    w.end_box(trex);
    w.end_box(s);
}

// ---------------------------------------------------------------------------
// Media segment (styp + moof + mdat)
// ---------------------------------------------------------------------------

/// Build one fMP4 media segment containing `frames` AAC payloads, each
/// `sample_duration` samples long (typically 1024 for AAC-LC) and starting
/// at `base_media_decode_time` (cumulative sample count from stream start).
pub(crate) fn write_media_segment(
    seq: u32,
    base_media_decode_time: u64,
    sample_duration: u32,
    frames: &[Vec<u8>],
) -> Vec<u8> {
    let mut w = Writer::new();

    // styp
    let styp = w.begin_box(b"styp");
    w.fourcc(b"msdh");
    w.u32(0); // minor
    w.fourcc(b"msdh");
    w.fourcc(b"msix");
    w.end_box(styp);

    // We need the moof size in advance because trun's data_offset is
    // relative to the start of the moof and must point at the first
    // mdat sample. We build the trun with a placeholder data_offset,
    // close moof, then patch the offset.
    let moof_start = w.begin_box(b"moof");

    // mfhd
    let mfhd = w.begin_full_box(b"mfhd", 0, 0);
    w.u32(seq);
    w.end_box(mfhd);

    // traf
    let traf = w.begin_box(b"traf");

    // tfhd: track_id only, with default-base-is-moof flag (0x020000) so that
    // any data_offset in trun is relative to the moof.
    let tfhd = w.begin_full_box(b"tfhd", 0, 0x02_0000);
    w.u32(TRACK_ID);
    w.end_box(tfhd);

    // tfdt (version 1, 64-bit base_media_decode_time)
    let tfdt = w.begin_full_box(b"tfdt", 1, 0);
    w.u64(base_media_decode_time);
    w.end_box(tfdt);

    // trun
    // flags: data-offset-present(0x000001)
    //        + sample-duration-present(0x000100)
    //        + sample-size-present(0x000200)
    let trun_flags: u32 = 0x0001 | 0x0100 | 0x0200;
    let trun = w.begin_full_box(b"trun", 0, trun_flags);
    w.u32(frames.len() as u32); // sample_count
    let data_offset_pos = w.len();
    w.u32(0); // data_offset placeholder (patched later)
    for f in frames {
        w.u32(sample_duration);
        w.u32(f.len() as u32);
    }
    w.end_box(trun);

    w.end_box(traf);
    w.end_box(moof_start);

    let moof_size = w.len() - moof_start;
    // data_offset = moof_size + 8 (mdat header)
    let data_offset = (moof_size + 8) as u32;
    w.buf[data_offset_pos..data_offset_pos + 4].copy_from_slice(&data_offset.to_be_bytes());

    // mdat
    let mdat = w.begin_box(b"mdat");
    for f in frames {
        w.bytes(f);
    }
    w.end_box(mdat);

    w.into_vec()
}
