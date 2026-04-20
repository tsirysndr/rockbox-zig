// ALAC verbatim/escape frame encoder for 352 stereo 16-bit PCM samples.
//
// Layout matching David Hammerton's ALAC decoder (used by shairport-sync).
// The hammerton decoder does NOT parse AAC element-type tags from the bitstream;
// instead the first 3 bits are a "channel count" field (value 1 = stereo), and
// there is NO 4-bit element-instance field after it.
//
//   3 bits  channels = 0b001 (1 = stereo)
//   4 bits  output_waiting = 0   (read and discarded)
//  12 bits  unknown = 0          (read and discarded)
//   1 bit   hassize = 0
//   2 bits  uncompressed_bytes = 0
//   1 bit   isNotCompressed = 1   ← bit 22 from frame start
//   352 × (16-bit left, 16-bit right) — interleaved per sample
//   pad to byte boundary            (no END tag)
//   → total: 23 + 352×32 = 11287 bits → 1411 bytes

pub const FRAME_SAMPLES: usize = 352;
pub const PCM_BYTES_PER_FRAME: usize = FRAME_SAMPLES * 4; // stereo S16LE
pub const ALAC_FRAME_BYTES: usize = 1411;

struct BitWriter<'a> {
    buf: &'a mut [u8],
    pos: usize,
}

impl<'a> BitWriter<'a> {
    fn new(buf: &'a mut [u8]) -> Self {
        Self { buf, pos: 0 }
    }

    fn write(&mut self, value: u32, nbits: usize) {
        for i in (0..nbits).rev() {
            let bit = (value >> i) & 1;
            let byte_idx = self.pos / 8;
            let bit_idx = 7 - (self.pos % 8);
            if bit == 1 {
                self.buf[byte_idx] |= 1 << bit_idx;
            }
            self.pos += 1;
        }
    }

    fn align(&mut self) {
        if self.pos % 8 != 0 {
            self.pos = (self.pos + 7) & !7;
        }
    }
}

pub fn encode_frame(pcm: &[u8]) -> [u8; ALAC_FRAME_BYTES] {
    assert_eq!(pcm.len(), PCM_BYTES_PER_FRAME);
    let mut out = [0u8; ALAC_FRAME_BYTES];
    let mut w = BitWriter::new(&mut out);

    // 23-bit header — matches the exact readbits() sequence in hammerton alac.c:
    w.write(0b001, 3); // channels = 1 (stereo); decoder: readbits(3)
    w.write(0, 4); // output_waiting;          decoder: readbits(4) — discarded
    w.write(0, 12); // unknown;                  decoder: readbits(12) — discarded
    w.write(0, 1); // hassize = 0;              decoder: readbits(1)
    w.write(0, 2); // uncompressed_bytes = 0;   decoder: readbits(2)
    w.write(1, 1); // isNotCompressed = 1;      decoder: readbits(1) — bit 22

    // Interleaved samples: L0, R0, L1, R1, ...
    for i in 0..FRAME_SAMPLES {
        let base = i * 4;
        let l = i16::from_le_bytes([pcm[base], pcm[base + 1]]) as u16 as u32;
        let r = i16::from_le_bytes([pcm[base + 2], pcm[base + 3]]) as u16 as u32;
        w.write(l, 16);
        w.write(r, 16);
    }

    w.align(); // pad to byte boundary (hammerton has no END tag)

    out
}
