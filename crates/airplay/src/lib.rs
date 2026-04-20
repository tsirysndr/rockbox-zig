mod airplay2;
mod alac;
mod rtp;
mod rtsp;

// Called from rockbox-cli to force this crate's symbols into librockbox_cli.a
#[doc(hidden)]
pub fn _link_airplay() {}

use alac::{encode_frame, PCM_BYTES_PER_FRAME};
use rtp::RtpSender;
use rtsp::RtspClient;

use std::ffi::CStr;
use std::os::raw::{c_char, c_int, c_ushort};
use std::sync::Mutex;

static SESSION: Mutex<Option<AirPlaySession>> = Mutex::new(None);

struct AirPlaySession {
    sender: RtpSender,
    rtsp: RtspClient,
    buf: Vec<u8>,
    first_frame: bool,
}

static CONFIG: Mutex<AirPlayConfig> = Mutex::new(AirPlayConfig {
    host: None,
    port: 5000,
});

struct AirPlayConfig {
    host: Option<String>,
    port: u16,
}

// Safety: the raw pointer in host is only touched inside the mutex
unsafe impl Send for AirPlayConfig {}

#[no_mangle]
pub extern "C" fn pcm_airplay_set_host(host: *const c_char, port: c_ushort) {
    if host.is_null() {
        return;
    }
    let s = unsafe { CStr::from_ptr(host) }
        .to_string_lossy()
        .into_owned();
    let mut cfg = CONFIG.lock().unwrap();
    cfg.host = Some(s);
    cfg.port = port;
}

#[no_mangle]
pub extern "C" fn pcm_airplay_connect() -> c_int {
    // Already connected — don't redo the RTSP handshake for every DMA chunk.
    if SESSION.lock().unwrap().is_some() {
        return 0;
    }

    let cfg = CONFIG.lock().unwrap();
    let host = match cfg.host.clone() {
        Some(h) => h,
        None => {
            tracing::error!("pcm_airplay_connect: no host configured");
            return -1;
        }
    };
    let port = cfg.port;
    drop(cfg);

    let local_ip = local_ip_for(&host).unwrap_or_else(|| "127.0.0.1".to_string());
    tracing::info!("connecting to {}:{} (local_ip={})", host, port, local_ip);

    // Attempt AirPlay 2 pairing (PAIR-VERIFY / PAIR-SETUP).
    // Failure here is non-fatal — many AirPlay 1 receivers don't have the endpoint.
    match airplay2::connect(&host, port, None) {
        Ok(()) => tracing::info!("AirPlay 2 handshake complete"),
        Err(e) => tracing::debug!("AirPlay 2 handshake skipped ({}), using AirPlay 1", e),
    }

    let session_token: u64 = rand::random();
    let ssrc: u32 = rand::random();
    let initial_rtptime: u32 = rand::random();

    // Bind all UDP sockets first so we know the local ports before SETUP.
    let mut sender = match RtpSender::bind(ssrc, initial_rtptime) {
        Ok(s) => s,
        Err(e) => {
            tracing::error!("bind failed: {}", e);
            return -1;
        }
    };
    let local_ctrl_port = sender.local_ctrl_port;
    let local_timing_port = sender.local_timing_port;

    let mut rtsp = match RtspClient::connect(&host, port, session_token) {
        Ok(c) => c,
        Err(e) => {
            tracing::error!("RTSP TCP connect failed: {}", e);
            return -1;
        }
    };

    if let Err(e) = rtsp.announce(&local_ip, &host) {
        tracing::error!("ANNOUNCE failed: {}", e);
        return -1;
    }

    let (server_audio, server_ctrl, _server_timing) =
        match rtsp.setup(local_ctrl_port, local_timing_port) {
            Ok(ports) => ports,
            Err(e) => {
                tracing::error!("SETUP failed: {}", e);
                return -1;
            }
        };

    if let Err(e) = sender.connect_server(&host, server_audio, server_ctrl) {
        tracing::error!("connect_server failed: {}", e);
        return -1;
    }

    if let Err(e) = rtsp.record(0, initial_rtptime) {
        tracing::error!("RECORD failed: {}", e);
        return -1;
    }

    // Set volume to maximum; RAOP range: -144.0 (mute) to 0.0 (full).
    if let Err(e) = rtsp.set_parameter_volume(0.0) {
        tracing::warn!("SET_PARAMETER volume failed (non-fatal): {}", e);
    }

    sender.send_initial_sync();
    tracing::info!(
        "session established — sending audio to {}:{}",
        host,
        server_audio
    );

    let mut guard = SESSION.lock().unwrap();
    *guard = Some(AirPlaySession {
        sender,
        rtsp,
        buf: Vec::with_capacity(PCM_BYTES_PER_FRAME * 4),
        first_frame: true,
    });

    0
}

/// Write raw S16LE stereo PCM. Buffers into 352-sample frames, encodes ALAC, sends RTP.
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
        tracing::debug!("first write: {} bytes", len);
    }

    session.buf.extend_from_slice(input);

    while session.buf.len() >= PCM_BYTES_PER_FRAME {
        let frame_bytes: [u8; PCM_BYTES_PER_FRAME] =
            session.buf[..PCM_BYTES_PER_FRAME].try_into().unwrap();
        session.buf.drain(..PCM_BYTES_PER_FRAME);

        let alac = encode_frame(&frame_bytes);
        let first = session.first_frame;
        session.first_frame = false;
        session.sender.send_audio(&alac, first);
    }

    0
}

#[no_mangle]
pub extern "C" fn pcm_airplay_stop() {
    let mut guard = SESSION.lock().unwrap();
    if let Some(ref mut session) = *guard {
        let _ = session.rtsp.teardown();
    }
    *guard = None;
}

#[no_mangle]
pub extern "C" fn pcm_airplay_close() {
    let mut guard = SESSION.lock().unwrap();
    if let Some(ref mut session) = *guard {
        let _ = session.rtsp.teardown();
    }
    *guard = None;
}

fn local_ip_for(remote: &str) -> Option<String> {
    use std::net::UdpSocket;
    let sock = UdpSocket::bind("0.0.0.0:0").ok()?;
    sock.connect(format!("{}:80", remote)).ok()?;
    sock.local_addr().ok().map(|a| a.ip().to_string())
}
