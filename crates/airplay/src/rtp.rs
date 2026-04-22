use std::net::UdpSocket;
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use crate::alac::{ALAC_FRAME_BYTES, FRAME_SAMPLES};

const RTP_HEADER_BYTES: usize = 12;
pub const RTP_PACKET_BYTES: usize = RTP_HEADER_BYTES + ALAC_FRAME_BYTES;

const NTP_EPOCH_DELTA: u32 = 0x83AA_7E80;

// Duration of one ALAC frame at 44100 Hz
pub const FRAME_DURATION_US: u64 = FRAME_SAMPLES as u64 * 1_000_000 / 44100; // ~7982 µs

/// Per-receiver UDP state. One of these per AirPlay endpoint.
pub struct ReceiverHandle {
    audio_sock: UdpSocket,
    ctrl_sock: UdpSocket,
    server_ctrl_addr: std::net::SocketAddr,
    pub local_ctrl_port: u16,
    pub ssrc: u32,
    pub seqnum: u16,
}

impl ReceiverHandle {
    /// Bind local audio and ctrl sockets. Call `connect()` after SETUP.
    pub fn bind() -> std::io::Result<Self> {
        let audio_sock = UdpSocket::bind("0.0.0.0:0")?;
        let ctrl_sock = UdpSocket::bind("0.0.0.0:0")?;
        let local_ctrl_port = ctrl_sock.local_addr()?.port();
        let ssrc: u32 = rand::random();
        let server_ctrl_addr = "0.0.0.0:0".parse().unwrap();
        Ok(Self {
            audio_sock,
            ctrl_sock,
            server_ctrl_addr,
            local_ctrl_port,
            ssrc,
            seqnum: 0,
        })
    }

    /// Connect the audio socket to the server's RTP port and record the ctrl addr.
    pub fn connect(&mut self, host: &str, audio_port: u16, ctrl_port: u16) -> std::io::Result<()> {
        tracing::debug!("connecting audio → {}:{}", host, audio_port);
        self.audio_sock
            .connect(format!("{}:{}", host, audio_port))?;
        self.server_ctrl_addr = format!("{}:{}", host, ctrl_port)
            .parse()
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidInput, e))?;
        Ok(())
    }

    /// Build and send one RTP audio packet. Increments seqnum.
    pub fn send_audio_packet(
        &mut self,
        alac_frame: &[u8; ALAC_FRAME_BYTES],
        rtptime: u32,
        frame_index: u64,
        first: bool,
    ) {
        let mut pkt = [0u8; RTP_PACKET_BYTES];
        pkt[0] = 0x80;
        pkt[1] = if first { 0x60 | 0x80 } else { 0x60 }; // M=1 on first, PT=96
        pkt[2] = (self.seqnum >> 8) as u8;
        pkt[3] = self.seqnum as u8;
        pkt[4] = (rtptime >> 24) as u8;
        pkt[5] = (rtptime >> 16) as u8;
        pkt[6] = (rtptime >> 8) as u8;
        pkt[7] = rtptime as u8;
        pkt[8] = (self.ssrc >> 24) as u8;
        pkt[9] = (self.ssrc >> 16) as u8;
        pkt[10] = (self.ssrc >> 8) as u8;
        pkt[11] = self.ssrc as u8;
        pkt[RTP_HEADER_BYTES..].copy_from_slice(alac_frame);

        match self.audio_sock.send(&pkt) {
            Ok(_) => {
                if frame_index < 5 {
                    tracing::debug!(
                        "sent frame {} ts={} seq={} first={}",
                        frame_index,
                        rtptime,
                        self.seqnum,
                        first
                    );
                }
            }
            Err(e) => tracing::warn!("send error on frame {}: {}", frame_index, e),
        }
        self.seqnum = self.seqnum.wrapping_add(1);
    }

    /// Send an RTCP NTP sync packet on the ctrl socket.
    pub fn send_sync(&self, current_ts: u32, next_ts: u32, first: bool) {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default();
        let ntp_sec = now.as_secs() as u32 + NTP_EPOCH_DELTA;
        let ntp_frac = ((now.subsec_nanos() as u64 * (1u64 << 32)) / 1_000_000_000) as u32;

        let mut pkt = [0u8; 20];
        pkt[0] = if first { 0x90 } else { 0x80 };
        pkt[1] = 0xd4;
        pkt[2] = 0x00;
        pkt[3] = 0x07;
        pkt[4] = (current_ts >> 24) as u8;
        pkt[5] = (current_ts >> 16) as u8;
        pkt[6] = (current_ts >> 8) as u8;
        pkt[7] = current_ts as u8;
        pkt[8] = (ntp_sec >> 24) as u8;
        pkt[9] = (ntp_sec >> 16) as u8;
        pkt[10] = (ntp_sec >> 8) as u8;
        pkt[11] = ntp_sec as u8;
        pkt[12] = (ntp_frac >> 24) as u8;
        pkt[13] = (ntp_frac >> 16) as u8;
        pkt[14] = (ntp_frac >> 8) as u8;
        pkt[15] = ntp_frac as u8;
        pkt[16] = (next_ts >> 24) as u8;
        pkt[17] = (next_ts >> 16) as u8;
        pkt[18] = (next_ts >> 8) as u8;
        pkt[19] = next_ts as u8;

        let _ = self.ctrl_sock.send_to(&pkt, self.server_ctrl_addr);
    }
}

/// Shared NTP timing socket. One instance serves all receivers — the responder
/// doesn't care which receiver the request came from, it just replies in place.
pub struct TimingSocket {
    pub local_port: u16,
    // kept alive so the OS port stays open; responder thread holds the other Arc
    _sock: Arc<UdpSocket>,
}

impl TimingSocket {
    pub fn bind() -> std::io::Result<Self> {
        let sock = Arc::new(UdpSocket::bind("0.0.0.0:0")?);
        let local_port = sock.local_addr()?.port();
        let thread_sock = Arc::clone(&sock);
        thread::spawn(move || timing_responder(thread_sock));
        tracing::debug!("timing responder bound on port {}", local_port);
        Ok(Self {
            local_port,
            _sock: sock,
        })
    }
}

/// Pacing state shared across all receivers.
pub struct PacingClock {
    pub stream_start: Option<Instant>,
    pub frames_sent: u64,
    pub rtptime: u32,
    pub initial_rtptime: u32,
}

impl PacingClock {
    pub fn new(initial_rtptime: u32) -> Self {
        Self {
            stream_start: None,
            frames_sent: 0,
            rtptime: initial_rtptime,
            initial_rtptime,
        }
    }

    /// Advance after sending one frame to all receivers.
    pub fn advance(&mut self) {
        self.rtptime = self.rtptime.wrapping_add(FRAME_SAMPLES as u32);
        self.frames_sent += 1;
    }

    /// Sleep until the current frame's real-time deadline.
    pub fn pace(&mut self) {
        let start = *self.stream_start.get_or_insert_with(Instant::now);
        let expected = start + Duration::from_micros(self.frames_sent * FRAME_DURATION_US);
        let now = Instant::now();
        if expected > now {
            std::thread::sleep(expected - now);
        }
    }

    pub fn reset(&mut self) {
        self.stream_start = None;
        self.frames_sent = 0;
        self.rtptime = self.initial_rtptime;
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
            tracing::debug!(
                "timing: unexpected packet len={} type=0x{:02X} from {}",
                len,
                buf[1],
                src
            );
            continue;
        }
        tracing::debug!("timing request from {}", src);

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default();
        let ntp_sec = now.as_secs() as u32 + NTP_EPOCH_DELTA;
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
        resp[26] = (ntp_sec >> 8) as u8;
        resp[27] = ntp_sec as u8;
        resp[28] = (ntp_frac >> 24) as u8;
        resp[29] = (ntp_frac >> 16) as u8;
        resp[30] = (ntp_frac >> 8) as u8;
        resp[31] = ntp_frac as u8;

        let _ = sock.send_to(&resp, src);
    }
}
