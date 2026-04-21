use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{mpsc, Arc, Mutex};

/// Slim Protocol TCP server.  Each squeezelite instance that connects gets a
/// STRM command pointing at our HTTP broadcast endpoint.  Multiple clients are
/// fully supported — each receives an independent BroadcastReceiver cursor into
/// the shared PCM buffer plus a per-second `sync` command so all instances
/// align to the same server jiffies reference.
pub fn serve(slim_port: u16, http_port: u16) {
    let listener = match TcpListener::bind(("0.0.0.0", slim_port)) {
        Ok(l) => l,
        Err(e) => {
            tracing::error!("slim: bind :{slim_port} failed: {e}");
            return;
        }
    };
    tracing::info!("slim: listening on :{slim_port}");

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                // Subscribe before spawning so the sender is registered
                // before the first sync broadcast fires.
                let sync_rx = crate::get_sync().subscribe();
                std::thread::spawn(move || handle_client(stream, http_port, sync_rx));
            }
            Err(e) => tracing::warn!("slim: accept error: {e}"),
        }
    }
}

fn handle_client(mut stream: TcpStream, http_port: u16, sync_rx: mpsc::Receiver<u32>) {
    let peer = stream
        .peer_addr()
        .map(|a| a.to_string())
        .unwrap_or_default();
    tracing::info!("slim: client connected from {peer}");

    match read_client_packet(&mut stream) {
        Ok((opcode, _body)) if opcode == "HELO" => {
            tracing::info!("slim: HELO from {peer}");
        }
        Ok((opcode, _)) => {
            tracing::warn!("slim: expected HELO, got '{opcode}' from {peer}");
            return;
        }
        Err(e) => {
            tracing::debug!("slim: read error from {peer}: {e}");
            return;
        }
    }

    if let Err(e) = send_strm_start(&mut stream, http_port) {
        tracing::error!("slim: send STRM to {peer} failed: {e}");
        return;
    }
    tracing::info!("slim: sent STRM to {peer} → http stream on :{http_port}");

    // Clone the stream for writes; reads stay on the original fd.
    // Both fds refer to the same socket — POSIX guarantees this is safe.
    let write_stream = match stream.try_clone() {
        Ok(s) => Arc::new(Mutex::new(s)),
        Err(e) => {
            tracing::error!("slim: try_clone failed for {peer}: {e}");
            return;
        }
    };

    // Sync writer thread: receives jiffies from the broadcaster and forwards
    // `sync` packets to this client so it aligns with the server clock.
    // Runs concurrently with the read loop below, sharing write_stream.
    {
        let ws = Arc::clone(&write_stream);
        let peer_label = peer.clone();
        std::thread::spawn(move || {
            for jiffies in sync_rx {
                let mut s = ws.lock().unwrap();
                if let Err(e) = send_sync(&mut *s, jiffies) {
                    tracing::debug!("slim: sync write error to {peer_label}: {e}");
                    break;
                }
                tracing::debug!("slim: sync jiffies={jiffies} → {peer_label}");
            }
        });
    }

    // Read loop: handle STAT / DSCO packets.
    // Reply to every STMt heartbeat with `audg` to keep squeezelite's 36-second
    // watchdog from firing.  Log timing data for diagnostics.
    loop {
        match read_client_packet(&mut stream) {
            Ok((opcode, body)) => {
                if opcode == "STAT" && body.len() >= 4 {
                    let ev = std::str::from_utf8(&body[..4]).unwrap_or("????");
                    if ev == "STMt" {
                        let elapsed_ms = stmt_elapsed_ms(&body);
                        let client_jiffies = stmt_jiffies(&body);
                        tracing::debug!(
                            "slim: STMt from {peer}: elapsed={elapsed_ms}ms \
                             client_jiffies={client_jiffies}"
                        );
                        let mut s = write_stream.lock().unwrap();
                        if let Err(e) = send_audg(&mut *s) {
                            tracing::debug!("slim: audg error to {peer}: {e}");
                            break;
                        }
                    } else {
                        tracing::debug!("slim: STAT {ev} from {peer}");
                    }
                } else if opcode == "DSCO" {
                    tracing::info!("slim: DSCO from {peer}");
                    break;
                } else {
                    tracing::debug!("slim: {opcode} from {peer}");
                }
            }
            Err(_) => {
                tracing::info!("slim: {peer} disconnected");
                break;
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Packet I/O helpers
// ---------------------------------------------------------------------------

fn read_client_packet(stream: &mut TcpStream) -> std::io::Result<(String, Vec<u8>)> {
    let mut opcode = [0u8; 4];
    stream.read_exact(&mut opcode)?;

    let mut len_buf = [0u8; 4];
    stream.read_exact(&mut len_buf)?;
    let len = u32::from_be_bytes(len_buf) as usize;

    let mut payload = vec![0u8; len];
    if len > 0 {
        stream.read_exact(&mut payload)?;
    }
    Ok((String::from_utf8_lossy(&opcode).into_owned(), payload))
}

fn send_server_packet(
    stream: &mut TcpStream,
    opcode: &[u8; 4],
    payload: &[u8],
) -> std::io::Result<()> {
    let total = 4usize + payload.len();
    stream.write_all(&(total as u16).to_be_bytes())?;
    stream.write_all(opcode)?;
    stream.write_all(payload)?;
    Ok(())
}

fn send_strm_start(stream: &mut TcpStream, http_port: u16) -> std::io::Result<()> {
    let request = b"GET /stream.pcm HTTP/1.0\r\n\r\n";
    let mut payload = Vec::with_capacity(24 + request.len());
    payload.push(b's'); // command: start
    payload.push(b'1'); // autostart
    payload.push(b'p'); // format: raw PCM
    payload.push(b'1'); // pcm_sample_size: 16-bit  (squeezelite: field - '0')
    payload.push(b'3'); // pcm_sample_rate: 44100   (squeezelite: field - '0')
    payload.push(b'2'); // pcm_channels: stereo     (squeezelite: field - '0')
    payload.push(b'1'); // pcm_endianness: LE        (squeezelite: field - '0')
    payload.push(255u8); // threshold: 255 KB
    payload.push(0u8); // spdif_enable
    payload.push(0u8); // transition_period
    payload.push(b'0'); // transition_type: none
    payload.push(0u8); // flags
    payload.push(0u8); // output_threshold
    payload.push(0u8); // slaves
    payload.extend_from_slice(&0x00010000u32.to_be_bytes()); // replay_gain = 1.0
    payload.extend_from_slice(&http_port.to_be_bytes());
    payload.extend_from_slice(&0u32.to_be_bytes()); // server_ip = 0 → use slimproto IP
    payload.extend_from_slice(request);
    send_server_packet(stream, b"strm", &payload)
}

/// `audg` — full-volume gain packet; sent on every STMt heartbeat to suppress
/// squeezelite's 36-second "no messages from server" watchdog.
fn send_audg(stream: &mut TcpStream) -> std::io::Result<()> {
    let mut payload = [0u8; 9];
    payload[0..4].copy_from_slice(&0x00010000u32.to_be_bytes()); // left  gain = 1.0
    payload[4..8].copy_from_slice(&0x00010000u32.to_be_bytes()); // right gain = 1.0
    send_server_packet(stream, b"audg", &payload)
}

/// `sync` — tells squeezelite to align its playback clock to `jiffies`.
/// All clients receive the same value from the broadcaster, causing them to
/// converge to the same audio position.
fn send_sync(stream: &mut TcpStream, jiffies: u32) -> std::io::Result<()> {
    send_server_packet(stream, b"sync", &jiffies.to_be_bytes())
}

// ---------------------------------------------------------------------------
// STMt body parsers
//
// STMt body layout (all fields big-endian):
//   [0..4]   event ("STMt")
//   [4]      num_crlf
//   [5]      mas_initialized
//   [6]      mas_mode
//   [7..11]  rptr  (stream buffer read pointer)
//   [11..15] wptr  (stream buffer write pointer)
//   [15..23] bytes_received (u64)
//   [23..25] signal_strength (u16)
//   [25..29] jiffies (u32)           ← client's monotonic ms clock
//   [29..33] output_buffer_size
//   [33..37] output_buffer_fullness
//   [37..41] elapsed_seconds
//   [41..43] voltage (u16)
//   [43..47] elapsed_milliseconds    ← ms of audio output so far
//   [47..51] server_timestamp        (echo of last strm timestamp)
//   [51..53] error_code (u16)
// ---------------------------------------------------------------------------

fn stmt_jiffies(body: &[u8]) -> u32 {
    read_u32_be(body, 25)
}

fn stmt_elapsed_ms(body: &[u8]) -> u32 {
    read_u32_be(body, 43)
}

fn read_u32_be(data: &[u8], offset: usize) -> u32 {
    if data.len() < offset + 4 {
        return 0;
    }
    u32::from_be_bytes([
        data[offset],
        data[offset + 1],
        data[offset + 2],
        data[offset + 3],
    ])
}
