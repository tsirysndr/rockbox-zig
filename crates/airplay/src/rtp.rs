use std::net::UdpSocket;
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use crate::alac::{ALAC_FRAME_BYTES, FRAME_SAMPLES};

const RTP_HEADER_BYTES: usize = 12;
pub const RTP_PACKET_BYTES: usize = RTP_HEADER_BYTES + ALAC_FRAME_BYTES;

const NTP_EPOCH_DELTA: u32 = 0x83AA_7E80;

// Duration of one ALAC frame at 44100 Hz
const FRAME_DURATION_US: u64 = FRAME_SAMPLES as u64 * 1_000_000 / 44100; // ~7982 µs

pub struct RtpSender {
    audio_sock: UdpSocket,
    ctrl_sock: UdpSocket,
    server_ctrl_addr: std::net::SocketAddr,
    ssrc: u32,
    seqnum: u16,
    rtptime: u32,
    initial_rtptime: u32,
    frames_sent: u64,
    pub local_ctrl_port: u16,
    pub local_timing_port: u16,
    stream_start: Option<Instant>,
    // kept alive so the OS port stays open; responder thread holds the other Arc
    _timing_sock: Arc<UdpSocket>,
}

impl RtpSender {
    /// Bind all local UDP sockets. `connect_server()` must be called after SETUP
    /// once the server's ports are known.
    pub fn bind(ssrc: u32, initial_rtptime: u32) -> std::io::Result<Self> {
        let audio_sock = UdpSocket::bind("0.0.0.0:0")?;
        let ctrl_sock = UdpSocket::bind("0.0.0.0:0")?;
        let local_ctrl_port = ctrl_sock.local_addr()?.port();
        let timing_sock = Arc::new(UdpSocket::bind("0.0.0.0:0")?);
        let local_timing_port = timing_sock.local_addr()?.port();

        // Respond to NTP timing requests from the receiver so it can synchronise
        // and actually start playing. Without this, the timing port gets ICMP
        // unreachable replies and many receivers stall indefinitely.
        let timing_thread = Arc::clone(&timing_sock);
        thread::spawn(move || timing_responder(timing_thread));

        let server_ctrl_addr = "0.0.0.0:0".parse().unwrap();
        tracing::debug!("local ctrl_port={} timing_port={}", local_ctrl_port, local_timing_port);

        Ok(Self {
            audio_sock,
            ctrl_sock,
            server_ctrl_addr,
            ssrc,
            seqnum: 0,
            rtptime: initial_rtptime,
            initial_rtptime,
            frames_sent: 0,
            local_ctrl_port,
            local_timing_port,
            stream_start: None,
            _timing_sock: timing_sock,
        })
    }

    /// Connect the audio socket to the server's RTP port and record the ctrl addr.
    pub fn connect_server(&mut self, host: &str, audio_port: u16, ctrl_port: u16) -> std::io::Result<()> {
        tracing::debug!("connecting audio → {}:{}", host, audio_port);
        self.audio_sock.connect(format!("{}:{}", host, audio_port))?;
        self.server_ctrl_addr = format!("{}:{}", host, ctrl_port)
            .parse()
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidInput, e))?;
        Ok(())
    }

    pub fn send_audio(&mut self, alac_frame: &[u8; ALAC_FRAME_BYTES], first: bool) {
        let start = *self.stream_start.get_or_insert_with(Instant::now);

        let mut pkt = [0u8; RTP_PACKET_BYTES];
        pkt[0] = 0x80;
        pkt[1] = if first { 0x60 | 0x80 } else { 0x60 }; // M=1 on first, PT=96
        pkt[2] = (self.seqnum >> 8) as u8;
        pkt[3] = self.seqnum as u8;
        pkt[4] = (self.rtptime >> 24) as u8;
        pkt[5] = (self.rtptime >> 16) as u8;
        pkt[6] = (self.rtptime >> 8) as u8;
        pkt[7] = self.rtptime as u8;
        pkt[8]  = (self.ssrc >> 24) as u8;
        pkt[9]  = (self.ssrc >> 16) as u8;
        pkt[10] = (self.ssrc >> 8) as u8;
        pkt[11] = self.ssrc as u8;
        pkt[RTP_HEADER_BYTES..].copy_from_slice(alac_frame);

        match self.audio_sock.send(&pkt) {
            Ok(_) => {
                if self.frames_sent < 5 {
                    tracing::debug!("sent frame {} ts={} seq={} first={}",
                              self.frames_sent, self.rtptime, self.seqnum, first);
                }
            }
            Err(e) => tracing::warn!("send error on frame {}: {}", self.frames_sent, e),
        }

        self.seqnum = self.seqnum.wrapping_add(1);
        self.rtptime = self.rtptime.wrapping_add(FRAME_SAMPLES as u32);
        self.frames_sent += 1;

        // RTCP NTP sync every ~44 frames (~0.35 s)
        if self.frames_sent % 44 == 0 {
            self.send_sync(false);
        }

        // Real-time pacing — sleep until the frame's playout deadline.
        let expected = start + Duration::from_micros(self.frames_sent * FRAME_DURATION_US);
        let now = Instant::now();
        if expected > now {
            std::thread::sleep(expected - now);
        }
    }

    fn send_sync(&self, first: bool) {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default();
        let ntp_sec  = now.as_secs() as u32 + NTP_EPOCH_DELTA;
        let ntp_frac = ((now.subsec_nanos() as u64 * (1u64 << 32)) / 1_000_000_000) as u32;

        // "current" timestamp = frame we just sent (rtptime was already incremented)
        let current_ts = self.rtptime.wrapping_sub(FRAME_SAMPLES as u32);
        // "next" timestamp = self.rtptime (next frame to be sent)
        let next_ts = self.rtptime;

        let mut pkt = [0u8; 20];
        pkt[0] = if first { 0x90 } else { 0x80 };
        pkt[1] = 0xd4;
        pkt[2] = 0x00;
        pkt[3] = 0x07;
        pkt[4]  = (current_ts >> 24) as u8;
        pkt[5]  = (current_ts >> 16) as u8;
        pkt[6]  = (current_ts >> 8)  as u8;
        pkt[7]  =  current_ts        as u8;
        pkt[8]  = (ntp_sec >> 24)    as u8;
        pkt[9]  = (ntp_sec >> 16)    as u8;
        pkt[10] = (ntp_sec >> 8)     as u8;
        pkt[11] =  ntp_sec           as u8;
        pkt[12] = (ntp_frac >> 24)   as u8;
        pkt[13] = (ntp_frac >> 16)   as u8;
        pkt[14] = (ntp_frac >> 8)    as u8;
        pkt[15] =  ntp_frac          as u8;
        pkt[16] = (next_ts >> 24)    as u8;
        pkt[17] = (next_ts >> 16)    as u8;
        pkt[18] = (next_ts >> 8)     as u8;
        pkt[19] =  next_ts           as u8;

        let _ = self.ctrl_sock.send_to(&pkt, self.server_ctrl_addr);
    }

    pub fn send_initial_sync(&self) {
        // At startup no frames have been sent yet; use initial_rtptime for both
        // "current" and "next" so we don't send a backwards-wrapped timestamp.
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default();
        let ntp_sec  = now.as_secs() as u32 + NTP_EPOCH_DELTA;
        let ntp_frac = ((now.subsec_nanos() as u64 * (1u64 << 32)) / 1_000_000_000) as u32;
        let ts = self.initial_rtptime;

        let mut pkt = [0u8; 20];
        pkt[0]  = 0x90; // first sync: extension bit set
        pkt[1]  = 0xd4;
        pkt[2]  = 0x00;
        pkt[3]  = 0x07;
        pkt[4]  = (ts >> 24) as u8;
        pkt[5]  = (ts >> 16) as u8;
        pkt[6]  = (ts >>  8) as u8;
        pkt[7]  =  ts        as u8;
        pkt[8]  = (ntp_sec  >> 24) as u8;
        pkt[9]  = (ntp_sec  >> 16) as u8;
        pkt[10] = (ntp_sec  >>  8) as u8;
        pkt[11] =  ntp_sec         as u8;
        pkt[12] = (ntp_frac >> 24) as u8;
        pkt[13] = (ntp_frac >> 16) as u8;
        pkt[14] = (ntp_frac >>  8) as u8;
        pkt[15] =  ntp_frac        as u8;
        pkt[16] = (ts >> 24) as u8;
        pkt[17] = (ts >> 16) as u8;
        pkt[18] = (ts >>  8) as u8;
        pkt[19] =  ts        as u8;

        let _ = self.ctrl_sock.send_to(&pkt, self.server_ctrl_addr);
        tracing::debug!("sent initial sync ts={}", ts);
    }

    pub fn reset_clock(&mut self) {
        self.stream_start = None;
        self.frames_sent = 0;
    }
}

// RAOP NTP timing responder.
//
// The receiver sends 32-byte timing requests (PT=0xD2) on the timing port we
// advertised in SETUP. We reply with PT=0xD3 so it can synchronise playback.
fn timing_responder(sock: Arc<UdpSocket>) {
    let mut buf = [0u8; 64];
    loop {
        let (len, src) = match sock.recv_from(&mut buf) {
            Ok(x) => x,
            Err(_) => break,
        };
        if len < 32 || buf[1] != 0xD2 {
            tracing::debug!("timing: unexpected packet len={} type=0x{:02X} from {}", len, buf[1], src);
            continue;
        }
        tracing::debug!("timing request from {}", src);

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default();
        let ntp_sec  = now.as_secs() as u32 + NTP_EPOCH_DELTA;
        let ntp_frac = ((now.subsec_nanos() as u64 * (1u64 << 32)) / 1_000_000_000) as u32;

        let mut resp = [0u8; 32];
        resp[0] = 0x80;
        resp[1] = 0xD3; // timing response
        resp[2] = buf[2]; // seq
        resp[3] = buf[3];
        // [8-15] reference — leave as zero
        // [16-23] originate = client's send time from request [16-23]
        resp[16..24].copy_from_slice(&buf[16..24]);
        // [24-31] receive + transmit = our current NTP
        resp[24] = (ntp_sec >> 24) as u8;
        resp[25] = (ntp_sec >> 16) as u8;
        resp[26] = (ntp_sec >> 8)  as u8;
        resp[27] =  ntp_sec        as u8;
        resp[28] = (ntp_frac >> 24) as u8;
        resp[29] = (ntp_frac >> 16) as u8;
        resp[30] = (ntp_frac >> 8)  as u8;
        resp[31] =  ntp_frac        as u8;

        let _ = sock.send_to(&resp, src);
    }
}
