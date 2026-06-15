use anyhow::{anyhow, Error};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;

pub async fn login(handle: &str) -> Result<(), Error> {
    let client = reqwest::Client::new();
    let res = client
        .post("https://api.rocksky.app/login")
        .json(&serde_json::json!({ "handle": handle, "cli": true }))
        .send()
        .await?;

    let redirect = res.text().await?;
    if !redirect.contains("authorize") {
        return Err(anyhow!("Login failed — try again"));
    }

    println!("Open this link in your browser to login:\n{}", redirect);
    open_url(&redirect);

    let token = receive_token().await?;

    let token_path = dirs::home_dir()
        .ok_or_else(|| anyhow!("home directory not found"))?
        .join(".config/rockbox.org/token");
    std::fs::create_dir_all(token_path.parent().unwrap())?;
    std::fs::write(&token_path, &token)?;

    println!("✅ Login successful!");
    Ok(())
}

/// Spin up a minimal HTTP server on port 6996 and wait for the OAuth callback
/// POST /token with body `{"token": "..."}`.
async fn receive_token() -> Result<String, Error> {
    let listener = TcpListener::bind("127.0.0.1:6996").await?;

    loop {
        let (mut stream, _) = listener.accept().await?;
        let mut buf = vec![0u8; 8192];
        let n = stream.read(&mut buf).await?;
        let request = String::from_utf8_lossy(&buf[..n]);

        let cors = "Access-Control-Allow-Origin: *\r\n\
                    Access-Control-Allow-Methods: POST\r\n\
                    Access-Control-Allow-Headers: content-type\r\n";

        // Handle CORS preflight
        if request.starts_with("OPTIONS") {
            let _ = stream
                .write_all(format!("HTTP/1.1 204 No Content\r\n{cors}\r\n").as_bytes())
                .await;
            continue;
        }

        // Parse the JSON body after the HTTP headers
        if let Some(body_start) = request.find("\r\n\r\n") {
            let body = request[body_start + 4..].trim();
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(body) {
                if let Some(token) = json["token"].as_str() {
                    let token = token.to_string();
                    let _ = stream
                        .write_all(
                            format!(
                                "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\n{cors}\r\n{{\"ok\":1}}"
                            )
                            .as_bytes(),
                        )
                        .await;
                    return Ok(token);
                }
            }
        }

        let _ = stream
            .write_all(
                format!("HTTP/1.1 400 Bad Request\r\n{cors}\r\n{{\"error\":\"missing token\"}}")
                    .as_bytes(),
            )
            .await;
    }
}

fn open_url(url: &str) {
    #[cfg(target_os = "macos")]
    let _ = std::process::Command::new("open").arg(url).spawn();

    #[cfg(target_os = "linux")]
    let _ = std::process::Command::new("xdg-open").arg(url).spawn();

    #[cfg(target_os = "windows")]
    let _ = std::process::Command::new("cmd")
        .args(["/c", "start", url])
        .spawn();
}
