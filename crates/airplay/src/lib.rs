mod airplay2;
mod alac;
mod rtp;
mod rtsp;

// Called from rockbox-cli to force this crate's symbols into librockbox_cli.a
#[doc(hidden)]
pub fn _link_airplay() {}

use alac::{encode_frame, FRAME_SAMPLES, PCM_BYTES_PER_FRAME};
use rtp::{PacingClock, ReceiverHandle, TimingSocket};
use rtsp::RtspClient;

use std::ffi::CStr;
use std::os::raw::{c_char, c_int, c_ushort};
use std::sync::Mutex;

// ---------------------------------------------------------------------------
// Global state
// ---------------------------------------------------------------------------

static SESSION: Mutex<Option<AirPlaySession>> = Mutex::new(None);

struct AirPlaySession {
    receivers: Vec<ReceiverHandle>,
    timing: TimingSocket,
    rtsp_clients: Vec<RtspClient>,
    buf: Vec<u8>,
    first_frame: bool,
    pacing: PacingClock,
}

impl AirPlaySession {
    /// Encode one ALAC frame and fan it out to every connected receiver.
    fn send_frame(&mut self, frame_bytes: &[u8; PCM_BYTES_PER_FRAME], first: bool) {
        let alac = encode_frame(frame_bytes);
        let rtptime = self.pacing.rtptime;
        let frame_index = self.pacing.frames_sent;

        for rx in &mut self.receivers {
            rx.send_audio_packet(&alac, rtptime, frame_index, first);
        }

        self.pacing.advance();

        // RTCP NTP sync every ~10 frames (~80 ms) for tighter multi-room alignment.
        // NTP offset = time until next_ts plays, so receivers anchor on the exact deadline.
        if self.pacing.frames_sent % 10 == 0 {
            let current_ts = self.pacing.rtptime.wrapping_sub(FRAME_SAMPLES as u32);
            let next_ts = self.pacing.rtptime;
            let offset_us = self.pacing.us_until_next_frame();
            for rx in &self.receivers {
                rx.send_sync(current_ts, next_ts, false, offset_us);
            }
        }

        // Pace once for all receivers
        self.pacing.pace();
    }

    fn send_initial_sync(&self) {
        let ts = self.pacing.initial_rtptime;
        for rx in &self.receivers {
            rx.send_sync(ts, ts, true, 0);
        }
        tracing::debug!(
            "sent initial sync ts={} to {} receiver(s)",
            ts,
            self.receivers.len()
        );
    }
}

// ---------------------------------------------------------------------------
// Config
// ---------------------------------------------------------------------------

static CONFIG: Mutex<AirPlayConfig> = Mutex::new(AirPlayConfig {
    receivers: Vec::new(),
});

struct AirPlayConfig {
    receivers: Vec<(String, u16)>,
}

// Safety: Vec<(String, u16)> is Send
unsafe impl Send for AirPlayConfig {}

// ---------------------------------------------------------------------------
// FFI — configuration
// ---------------------------------------------------------------------------

/// Set a single AirPlay receiver, replacing any previously configured list.
/// Kept for backward compatibility with existing C callers and settings.
#[no_mangle]
pub extern "C" fn pcm_airplay_set_host(host: *const c_char, port: c_ushort) {
    if host.is_null() {
        return;
    }
    let s = unsafe { CStr::from_ptr(host) }
        .to_string_lossy()
        .into_owned();
    let mut cfg = CONFIG.lock().unwrap();
    cfg.receivers.clear();
    cfg.receivers.push((s, port));
}

/// Append one receiver to the multi-room list.
#[no_mangle]
pub extern "C" fn pcm_airplay_add_receiver(host: *const c_char, port: c_ushort) {
    if host.is_null() {
        return;
    }
    let s = unsafe { CStr::from_ptr(host) }
        .to_string_lossy()
        .into_owned();
    let mut cfg = CONFIG.lock().unwrap();
    cfg.receivers.push((s, port));
}

/// Clear the receiver list (call before re-configuring).
#[no_mangle]
pub extern "C" fn pcm_airplay_clear_receivers() {
    CONFIG.lock().unwrap().receivers.clear();
}

// ---------------------------------------------------------------------------
// FFI — session lifecycle
// ---------------------------------------------------------------------------

#[no_mangle]
pub extern "C" fn pcm_airplay_connect() -> c_int {
    if SESSION.lock().unwrap().is_some() {
        return 0; // idempotent
    }

    let targets = {
        let cfg = CONFIG.lock().unwrap();
        if cfg.receivers.is_empty() {
            tracing::error!("pcm_airplay_connect: no receivers configured");
            return -1;
        }
        cfg.receivers.clone()
    };

    // All receivers share the same initial_rtptime for RTP-level synchronisation.
    let initial_rtptime: u32 = rand::random();

    // Bind the shared timing socket first; every receiver advertises the same port.
    let timing = match TimingSocket::bind() {
        Ok(t) => t,
        Err(e) => {
            tracing::error!("timing socket bind failed: {}", e);
            return -1;
        }
    };
    let local_timing_port = timing.local_port;

    let mut receivers: Vec<ReceiverHandle> = Vec::new();
    let mut rtsp_clients: Vec<RtspClient> = Vec::new();
    let mut connected = 0usize;

    for (host, port) in &targets {
        match connect_one(host, *port, initial_rtptime, local_timing_port) {
            Ok((rx, rtsp)) => {
                tracing::info!("connected to {}:{}", host, port);
                receivers.push(rx);
                rtsp_clients.push(rtsp);
                connected += 1;
            }
            Err(e) => tracing::warn!("failed to connect to {}:{}: {}", host, port, e),
        }
    }

    if connected == 0 {
        tracing::error!("could not connect to any AirPlay receiver");
        return -1;
    }

    let pacing = PacingClock::new(initial_rtptime);
    let session = AirPlaySession {
        receivers,
        timing,
        rtsp_clients,
        buf: Vec::with_capacity(PCM_BYTES_PER_FRAME * 4),
        first_frame: true,
        pacing,
    };

    session.send_initial_sync();
    tracing::info!(
        "session established: {}/{} receiver(s) connected",
        connected,
        targets.len()
    );

    *SESSION.lock().unwrap() = Some(session);
    0
}

/// Connect to a single AirPlay receiver. Returns `(ReceiverHandle, RtspClient)` on success.
fn connect_one(
    host: &str,
    port: u16,
    initial_rtptime: u32,
    local_timing_port: u16,
) -> std::io::Result<(ReceiverHandle, RtspClient)> {
    let local_ip = local_ip_for(host).unwrap_or_else(|| "127.0.0.1".to_string());

    // Attempt AirPlay 2 pairing (non-fatal fallback to AirPlay 1).
    match airplay2::connect(host, port, None) {
        Ok(()) => tracing::info!("AirPlay 2 handshake complete for {}:{}", host, port),
        Err(e) => tracing::debug!(
            "AirPlay 2 skipped for {}:{} ({}), using AirPlay 1",
            host,
            port,
            e
        ),
    }

    let session_token: u64 = rand::random();

    let mut rx = ReceiverHandle::bind()?;
    let local_ctrl_port = rx.local_ctrl_port;

    let mut rtsp = RtspClient::connect(host, port, session_token)?;
    rtsp.announce(&local_ip, host)?;

    let (server_audio, server_ctrl, _server_timing) =
        rtsp.setup(local_ctrl_port, local_timing_port)?;

    rx.connect(host, server_audio, server_ctrl)?;
    rtsp.record(0, initial_rtptime)?;

    // Set volume to maximum; RAOP range: -144.0 (mute) to 0.0 (full).
    if let Err(e) = rtsp.set_parameter_volume(0.0) {
        tracing::warn!(
            "SET_PARAMETER volume failed for {}:{} (non-fatal): {}",
            host,
            port,
            e
        );
    }

    Ok((rx, rtsp))
}

/// Write raw S16LE stereo PCM. Buffers into 352-sample frames, encodes ALAC,
/// fans out to every connected receiver, then paces once.
#[no_mangle]
pub extern "C" fn pcm_airplay_write(data: *const u8, len: usize) -> c_int {
    if data.is_null() || len == 0 {
        return 0;
    }
    let input = unsafe { std::slice::from_raw_parts(data, len) };

    let mut guard = SESSION.lock().unwrap();
    let session = match guard.as_mut() {
        Some(s) => s,
        None => {
            tracing::warn!("pcm_airplay_write: no session");
            return -1;
        }
    };

    if session.first_frame {
        tracing::debug!(
            "first write: {} bytes, {} receiver(s)",
            len,
            session.receivers.len()
        );
    }

    session.buf.extend_from_slice(input);

    while session.buf.len() >= PCM_BYTES_PER_FRAME {
        let frame_bytes: [u8; PCM_BYTES_PER_FRAME] =
            session.buf[..PCM_BYTES_PER_FRAME].try_into().unwrap();
        session.buf.drain(..PCM_BYTES_PER_FRAME);

        let first = session.first_frame;
        session.first_frame = false;
        session.send_frame(&frame_bytes, first);
    }

    0
}

#[no_mangle]
pub extern "C" fn pcm_airplay_stop() {
    let mut guard = SESSION.lock().unwrap();
    if let Some(ref mut session) = *guard {
        for rtsp in &mut session.rtsp_clients {
            let _ = rtsp.teardown();
        }
    }
    *guard = None;
}

#[no_mangle]
pub extern "C" fn pcm_airplay_close() {
    let mut guard = SESSION.lock().unwrap();
    if let Some(ref mut session) = *guard {
        for rtsp in &mut session.rtsp_clients {
            let _ = rtsp.teardown();
        }
    }
    *guard = None;
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn local_ip_for(remote: &str) -> Option<String> {
    use std::net::UdpSocket;
    let sock = UdpSocket::bind("0.0.0.0:0").ok()?;
    sock.connect(format!("{}:80", remote)).ok()?;
    sock.local_addr().ok().map(|a| a.ip().to_string())
}
