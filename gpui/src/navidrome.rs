use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct PingEnvelope {
    #[serde(rename = "subsonic-response")]
    response: PingBody,
}

#[derive(Debug, Deserialize)]
struct PingBody {
    status: String,
}

/// Generate a random 8-character alphanumeric salt.
pub fn random_salt() -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    use std::time::{SystemTime, UNIX_EPOCH};

    let mut h = DefaultHasher::new();
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .subsec_nanos()
        .hash(&mut h);
    std::thread::current().id().hash(&mut h);
    format!("{:016x}", h.finish())[..8].to_string()
}

/// Compute the Subsonic token: md5(password + salt).
pub fn compute_token(password: &str, salt: &str) -> String {
    let digest = md5::compute(format!("{}{}", password, salt));
    format!("{:x}", digest)
}

/// Verify credentials by calling /rest/ping.view.
/// Returns `true` if the server responds with `status: "ok"`.
pub async fn ping(base_url: &str, user: &str, token: &str, salt: &str) -> bool {
    let url = format!(
        "{}/rest/ping.view?u={}&t={}&s={}&v=1.16.1&c=rockbox&f=json",
        base_url.trim_end_matches('/'),
        user,
        token,
        salt
    );
    let client = match reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()
    {
        Ok(c) => c,
        Err(_) => return false,
    };
    match client.get(&url).send().await {
        Ok(resp) => {
            if !resp.status().is_success() {
                return false;
            }
            match resp.json::<PingEnvelope>().await {
                Ok(env) => env.response.status == "ok",
                Err(_) => false,
            }
        }
        Err(_) => false,
    }
}
