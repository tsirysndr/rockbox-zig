/// Authenticate with a Jellyfin API key.
/// Returns `Some((api_key, user_id))` on success.
pub async fn authenticate_with_api_key(base_url: &str, api_key: &str) -> Option<(String, String)> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .ok()?;
    let url = format!("{}/Users", base_url.trim_end_matches('/'));
    let resp = client
        .get(&url)
        .header("X-Emby-Token", api_key)
        .send()
        .await
        .ok()?;
    if !resp.status().is_success() {
        return None;
    }
    let users: Vec<serde_json::Value> = resp.json().await.ok()?;
    let user_id = users
        .into_iter()
        .next()
        .and_then(|u| u.get("Id").and_then(|id| id.as_str()).map(|s| s.to_string()))?;
    Some((api_key.to_string(), user_id))
}

/// Authenticate against a Jellyfin server.
///
/// Returns `Some((access_token, user_id))` on success, `None` on any error.
pub async fn authenticate(
    base_url: &str,
    username: &str,
    password: &str,
) -> Option<(String, String)> {
    let url = format!("{}/Users/AuthenticateByName", base_url.trim_end_matches('/'));
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(15))
        .build()
        .ok()?;

    let resp = client
        .post(&url)
        .header("Content-Type", "application/json")
        .header(
            "X-Emby-Authorization",
            r#"MediaBrowser Client="Rockbox", Device="Rockbox", DeviceId="rockbox-jellyfin", Version="1.0""#,
        )
        .json(&serde_json::json!({
            "Username": username,
            "Pw": password,
        }))
        .send()
        .await
        .ok()?;

    if !resp.status().is_success() {
        return None;
    }

    let body: serde_json::Value = resp.json().await.ok()?;
    let token = body.get("AccessToken")?.as_str()?.to_string();
    let user_id = body
        .get("User")
        .and_then(|u| u.get("Id"))
        .and_then(|id| id.as_str())?
        .to_string();

    Some((token, user_id))
}
