use std::time::Duration;

pub async fn ping(base_url: &str, user: Option<&str>, pass: Option<&str>) -> bool {
    let client = match reqwest::Client::builder()
        .timeout(Duration::from_secs(10))
        .build()
    {
        Ok(c) => c,
        Err(_) => return false,
    };
    let url = format!("{}/jsonrpc", base_url.trim_end_matches('/'));
    let body = serde_json::json!({
        "jsonrpc": "2.0",
        "method": "JSONRPC.Ping",
        "id": 1
    });
    let mut req = client.post(&url).json(&body);
    if let Some(u) = user {
        if !u.is_empty() {
            req = req.basic_auth(u, pass);
        }
    }
    match req.send().await {
        Ok(resp) => {
            if !resp.status().is_success() {
                return false;
            }
            match resp.json::<serde_json::Value>().await {
                Ok(v) => v["result"].as_str() == Some("pong"),
                Err(_) => false,
            }
        }
        Err(e) => {
            tracing::warn!("Kodi ping {url}: {e}");
            false
        }
    }
}
