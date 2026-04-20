use std::io::{self, BufRead, BufReader, Read, Write};
use std::net::TcpStream;

/// POST `path` with Content-Type: application/pairing+tlv8, return the body.
pub fn http_post_tlv8(host: &str, port: u16, path: &str, body: &[u8]) -> io::Result<Vec<u8>> {
    http_post(host, port, path, "application/pairing+tlv8", body)
}

/// POST `path` with `content_type`, return the response body.
pub fn http_post(
    host: &str,
    port: u16,
    path: &str,
    content_type: &str,
    body: &[u8],
) -> io::Result<Vec<u8>> {
    let mut stream = TcpStream::connect(format!("{}:{}", host, port))?;
    let req = format!(
        "POST {} HTTP/1.1\r\nHost: {}:{}\r\nContent-Type: {}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
        path, host, port, content_type, body.len()
    );
    stream.write_all(req.as_bytes())?;
    stream.write_all(body)?;
    stream.flush()?;

    let mut reader = BufReader::new(stream);

    // Parse status line
    let mut status_line = String::new();
    reader.read_line(&mut status_line)?;
    let status: u16 = status_line
        .split_whitespace()
        .nth(1)
        .and_then(|s| s.parse().ok())
        .unwrap_or(0);

    tracing::debug!("<< {}", status_line.trim());

    if status != 200 {
        return Err(io::Error::new(
            io::ErrorKind::Other,
            format!("HTTP {} returned status {}", path, status),
        ));
    }

    // Parse headers to get Content-Length
    let mut content_length: Option<usize> = None;
    loop {
        let mut line = String::new();
        reader.read_line(&mut line)?;
        let trimmed = line.trim_end_matches(|c| c == '\r' || c == '\n');
        if trimmed.is_empty() {
            break;
        }
        if let Some((k, v)) = trimmed.split_once(':') {
            if k.trim().to_lowercase() == "content-length" {
                content_length = v.trim().parse().ok();
            }
        }
    }

    // Read body
    let mut resp_body = Vec::new();
    if let Some(len) = content_length {
        resp_body.resize(len, 0);
        reader.read_exact(&mut resp_body)?;
    } else {
        reader.read_to_end(&mut resp_body)?;
    }

    Ok(resp_body)
}

/// GET `path`, return the response body.
pub fn http_get(host: &str, port: u16, path: &str) -> io::Result<Vec<u8>> {
    let mut stream = TcpStream::connect(format!("{}:{}", host, port))?;
    let req = format!(
        "GET {} HTTP/1.1\r\nHost: {}:{}\r\nConnection: close\r\n\r\n",
        path, host, port
    );
    stream.write_all(req.as_bytes())?;
    stream.flush()?;

    let mut reader = BufReader::new(stream);
    let mut status_line = String::new();
    reader.read_line(&mut status_line)?;
    let status: u16 = status_line
        .split_whitespace()
        .nth(1)
        .and_then(|s| s.parse().ok())
        .unwrap_or(0);

    if status != 200 {
        return Err(io::Error::new(
            io::ErrorKind::Other,
            format!("HTTP GET {} returned {}", path, status),
        ));
    }

    let mut content_length: Option<usize> = None;
    loop {
        let mut line = String::new();
        reader.read_line(&mut line)?;
        let trimmed = line.trim_end_matches(|c| c == '\r' || c == '\n');
        if trimmed.is_empty() {
            break;
        }
        if let Some((k, v)) = trimmed.split_once(':') {
            if k.trim().to_lowercase() == "content-length" {
                content_length = v.trim().parse().ok();
            }
        }
    }

    let mut resp_body = Vec::new();
    if let Some(len) = content_length {
        resp_body.resize(len, 0);
        reader.read_exact(&mut resp_body)?;
    } else {
        reader.read_to_end(&mut resp_body)?;
    }
    Ok(resp_body)
}
