use std::collections::HashMap;
use std::io::{self, BufRead, BufReader, Write as IoWrite};
use std::net::TcpStream;

pub struct RtspClient {
    stream: TcpStream,
    reader: BufReader<TcpStream>,
    cseq: u32,
    pub session_id: Option<String>,
    base_url: String,
    dacp_id: u64,
    active_remote: u32,
}

/// RTSP response: status code + headers.
struct Response {
    status: u16,
    headers: HashMap<String, String>,
}

impl RtspClient {
    pub fn connect(host: &str, port: u16, session_token: u64) -> io::Result<Self> {
        let stream = TcpStream::connect(format!("{}:{}", host, port))?;
        let reader = BufReader::new(stream.try_clone()?);
        // Use a plain decimal session id — some receivers choke on hex
        let base_url = format!("rtsp://{}:{}/{}", host, port, session_token);
        let dacp_id: u64 = rand::random();
        let active_remote: u32 = rand::random();
        Ok(Self { stream, reader, cseq: 0, session_id: None, base_url, dacp_id, active_remote })
    }

    fn send_request(
        &mut self,
        method: &str,
        uri: &str,
        headers: &[(&str, &str)],
        body: Option<&str>,
    ) -> io::Result<Response> {
        self.cseq += 1;
        let dacp_str = format!("{:016X}", self.dacp_id);
        let active_str = format!("{}", self.active_remote);
        let mut req = format!(
            "{} {} RTSP/1.0\r\nCSeq: {}\r\nUser-Agent: iTunes/12.0 (Rockbox)\r\nClient-Instance: {}\r\nDacp-ID: {}\r\nActive-Remote: {}\r\n",
            method, uri, self.cseq, dacp_str, dacp_str, active_str,
        );
        for (k, v) in headers {
            req.push_str(&format!("{}: {}\r\n", k, v));
        }
        if let Some(b) = body {
            req.push_str(&format!("Content-Length: {}\r\n", b.len()));
        }
        req.push_str("\r\n");
        if let Some(b) = body {
            req.push_str(b);
        }
        tracing::debug!(">> {} {}", method, uri);
        self.stream.write_all(req.as_bytes())?;
        self.stream.flush()?;
        self.read_response()
    }

    fn read_response(&mut self) -> io::Result<Response> {
        let mut status_line = String::new();
        self.reader.read_line(&mut status_line)?;
        let status_line = status_line.trim().to_string();
        tracing::debug!("<< {}", status_line);

        let status: u16 = status_line
            .split_whitespace()
            .nth(1)
            .and_then(|s| s.parse().ok())
            .unwrap_or(0);

        let mut headers = HashMap::new();
        loop {
            let mut line = String::new();
            self.reader.read_line(&mut line)?;
            let trimmed = line.trim_end_matches(|c| c == '\r' || c == '\n');
            if trimmed.is_empty() {
                break;
            }
            tracing::debug!("<< {}", trimmed);
            if let Some((key, val)) = trimmed.split_once(':') {
                headers.insert(key.trim().to_lowercase(), val.trim().to_string());
            }
        }

        // Consume body
        if let Some(len_str) = headers.get("content-length") {
            if let Ok(len) = len_str.parse::<usize>() {
                let mut body = vec![0u8; len];
                use std::io::Read;
                self.reader.read_exact(&mut body)?;
            }
        }

        Ok(Response { status, headers })
    }

    fn check_ok(resp: &Response, method: &str) -> io::Result<()> {
        if resp.status != 200 {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                format!("RTSP {} returned status {}", method, resp.status),
            ));
        }
        Ok(())
    }

    pub fn announce(&mut self, local_ip: &str, server_ip: &str) -> io::Result<()> {
        // m=audio 0: port 0 because the client is the sender; server port comes from SETUP
        let sdp = format!(
            "v=0\r\n\
             o=iTunes 3413821438 0 IN IP4 {local_ip}\r\n\
             s=iTunes\r\n\
             c=IN IP4 {server_ip}\r\n\
             t=0 0\r\n\
             m=audio 0 RTP/AVP 96\r\n\
             a=rtpmap:96 AppleLossless\r\n\
             a=fmtp:96 352 0 16 40 10 14 2 255 0 0 44100\r\n\
             a=min-latency:3528\r\n"
        );
        let url = self.base_url.clone();
        let resp = self.send_request("ANNOUNCE", &url, &[
            ("Content-Type", "application/sdp"),
        ], Some(&sdp))?;
        Self::check_ok(&resp, "ANNOUNCE")
    }

    /// Returns (server_audio_port, server_ctrl_port, server_timing_port).
    pub fn setup(&mut self, local_ctrl_port: u16, local_timing_port: u16) -> io::Result<(u16, u16, u16)> {
        // interleaved=0-1 is required by Apple receivers even for UDP transport.
        let transport = format!(
            "RTP/AVP/UDP;unicast;interleaved=0-1;mode=record;control_port={};timing_port={}",
            local_ctrl_port, local_timing_port
        );
        let url = self.base_url.clone();
        let resp = self.send_request("SETUP", &url, &[
            ("Transport", &transport),
        ], None)?;
        Self::check_ok(&resp, "SETUP")?;

        if let Some(sid) = resp.headers.get("session") {
            self.session_id = Some(sid.split(';').next().unwrap_or(sid).trim().to_string());
        }

        let transport_resp = resp.headers.get("transport").cloned().unwrap_or_default();
        let server_audio  = parse_port(&transport_resp, "server_port").unwrap_or(6000);
        let server_ctrl   = parse_port(&transport_resp, "control_port").unwrap_or(6001);
        let server_timing = parse_port(&transport_resp, "timing_port").unwrap_or(6002);
        tracing::debug!("server ports: audio={} ctrl={} timing={}", server_audio, server_ctrl, server_timing);
        Ok((server_audio, server_ctrl, server_timing))
    }

    pub fn record(&mut self, seqnum: u16, rtptime: u32) -> io::Result<()> {
        let rtp_info = format!("seq={};rtptime={}", seqnum, rtptime);
        let sid = self.session_id.clone().unwrap_or_default();
        let url = self.base_url.clone();
        let all_hdrs: Vec<(&str, &str)> = vec![
            ("Range", "npt=0-"),
            ("Session", &sid),
            ("RTP-Info", &rtp_info),
        ];
        let resp = self.send_request("RECORD", &url, &all_hdrs, None)?;
        Self::check_ok(&resp, "RECORD")
    }

    /// Send SET_PARAMETER with a `volume` value in RAOP range [-144.0, 0.0].
    /// 0.0 = full volume, -144.0 = mute.
    pub fn set_parameter_volume(&mut self, volume: f32) -> io::Result<()> {
        let body = format!("volume: {:.6}\r\n", volume);
        let sid = self.session_id.clone().unwrap_or_default();
        let url = self.base_url.clone();
        let resp = self.send_request("SET_PARAMETER", &url, &[
            ("Session", &sid),
            ("Content-Type", "text/parameters"),
        ], Some(&body))?;
        Self::check_ok(&resp, "SET_PARAMETER")
    }

    pub fn teardown(&mut self) -> io::Result<()> {
        let sid = self.session_id.clone().unwrap_or_default();
        let url = self.base_url.clone();
        let resp = self.send_request("TEARDOWN", &url, &[("Session", &sid)], None)?;
        Self::check_ok(&resp, "TEARDOWN")
    }
}

fn parse_port(transport: &str, key: &str) -> Option<u16> {
    for part in transport.split(';') {
        let part = part.trim();
        if let Some(rest) = part.strip_prefix(key) {
            let rest = rest.trim_start_matches('=');
            return rest.split('-').next()?.parse().ok();
        }
    }
    None
}
