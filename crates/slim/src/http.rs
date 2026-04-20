use crate::{BroadcastBuffer, RecvResult};
use std::io::Write;
use std::net::TcpListener;
use std::sync::Arc;

/// HTTP audio stream server.  Each accepted connection is handled in its own
/// thread and receives an independent BroadcastReceiver cursor into the shared
/// PCM buffer, so any number of squeezelite clients can play simultaneously.
pub fn serve(port: u16, buf: Arc<BroadcastBuffer>) {
    let listener = match TcpListener::bind(("0.0.0.0", port)) {
        Ok(l) => l,
        Err(e) => {
            tracing::error!("slim/http: bind :{port} failed: {e}");
            return;
        }
    };
    tracing::info!("slim/http: listening on :{port}");

    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
                let buf = buf.clone();
                std::thread::spawn(move || {
                    let peer = stream
                        .peer_addr()
                        .map(|a| a.to_string())
                        .unwrap_or_default();

                    if let Err(e) = drain_request(&mut stream) {
                        tracing::warn!("slim/http: request read error from {peer}: {e}");
                        return;
                    }

                    let headers = b"HTTP/1.0 200 OK\r\n\
                        Content-Type: audio/L16;rate=44100;channels=2\r\n\
                        Cache-Control: no-cache\r\n\
                        \r\n";
                    if let Err(e) = stream.write_all(headers) {
                        tracing::warn!("slim/http: header write error to {peer}: {e}");
                        return;
                    }

                    tracing::info!("slim/http: streaming PCM to {peer}");
                    let mut rx = buf.subscribe();

                    loop {
                        match rx.recv_blocking() {
                            RecvResult::Data(chunk) => {
                                if stream.write_all(&chunk).is_err() {
                                    tracing::debug!("slim/http: {peer} disconnected");
                                    break;
                                }
                            }
                            RecvResult::Closed => break,
                        }
                    }
                });
            }
            Err(e) => tracing::warn!("slim/http: accept error: {e}"),
        }
    }
}

fn drain_request(stream: &mut std::net::TcpStream) -> std::io::Result<()> {
    use std::io::Read;
    let mut buf: Vec<u8> = Vec::with_capacity(512);
    let mut byte = [0u8; 1];
    loop {
        stream.read_exact(&mut byte)?;
        buf.push(byte[0]);
        if buf.ends_with(b"\r\n\r\n") || buf.ends_with(b"\n\n") {
            return Ok(());
        }
        if buf.len() > 8192 {
            return Ok(());
        }
    }
}
