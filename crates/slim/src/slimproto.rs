use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};

/// Slim Protocol TCP server.  Each squeezelite instance that connects gets a
/// STRM command pointing at our HTTP broadcast endpoint.  Multiple clients are
/// fully supported — each connects to the same HTTP port and receives its own
/// independent read cursor into the shared PCM broadcast buffer.
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
                std::thread::spawn(move || handle_client(stream, http_port));
            }
            Err(e) => tracing::warn!("slim: accept error: {e}"),
        }
    }
}

fn handle_client(mut stream: TcpStream, http_port: u16) {
    let peer = stream.peer_addr().map(|a| a.to_string()).unwrap_or_default();
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

    // Read STAT / DSCO packets.  Reply to every STMt heartbeat with audg so
    // squeezelite's 36-second "no messages from server" watchdog never fires.
    loop {
        match read_client_packet(&mut stream) {
            Ok((opcode, body)) => {
                if opcode == "STAT" && body.len() >= 4 {
                    let ev = std::str::from_utf8(&body[..4]).unwrap_or("????");
                    tracing::debug!("slim: STAT {ev} from {peer}");
                    if ev == "STMt" {
                        if let Err(e) = send_audg(&mut stream) {
                            tracing::debug!("slim: audg error to {peer}: {e}");
                            break;
                        }
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
    payload.push(b's');           // command: start
    payload.push(b'1');           // autostart
    payload.push(b'p');           // format: raw PCM
    payload.push(b'1');           // pcm_sample_size: 16-bit
    payload.push(b'3');           // pcm_sample_rate: 44100 Hz
    payload.push(b'2');           // pcm_channels: stereo
    payload.push(b'1');           // pcm_endianness: little-endian
    payload.push(255u8);          // threshold: 255 KB
    payload.push(0u8);            // spdif_enable
    payload.push(0u8);            // transition_period
    payload.push(b'0');           // transition_type: none
    payload.push(0u8);            // flags
    payload.push(0u8);            // output_threshold
    payload.push(0u8);            // slaves
    payload.extend_from_slice(&0x00010000u32.to_be_bytes()); // replay_gain = 1.0
    payload.extend_from_slice(&http_port.to_be_bytes());
    payload.extend_from_slice(&0u32.to_be_bytes()); // server_ip = 0 → use slimproto_ip
    payload.extend_from_slice(request);
    send_server_packet(stream, b"strm", &payload)
}

fn send_audg(stream: &mut TcpStream) -> std::io::Result<()> {
    let mut payload = [0u8; 9];
    payload[0..4].copy_from_slice(&0x00010000u32.to_be_bytes()); // left  gain = 1.0
    payload[4..8].copy_from_slice(&0x00010000u32.to_be_bytes()); // right gain = 1.0
    send_server_packet(stream, b"audg", &payload)
}
