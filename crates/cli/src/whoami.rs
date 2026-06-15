use anyhow::Error;

use crate::settings::load_token;

pub async fn whoami() -> Result<(), Error> {
    let token = match load_token() {
        Ok(t) => t,
        Err(_) => {
            println!("You are not logged in. Use `rockboxd login <handle>` to login.");
            return Ok(());
        }
    };

    let client = reqwest::Client::new();
    let res = client
        .get("https://rocksky.app/profile")
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await?;

    if !res.status().is_success() {
        println!("You are not logged in. Use `rockboxd login <handle>` to login.");
        return Ok(());
    }

    let user = res.json::<serde_json::Value>().await?;
    println!(
        "You are logged in as: @{} ({})",
        user["handle"].as_str().unwrap_or("unknown"),
        user["displayName"].as_str().unwrap_or(""),
    );
    Ok(())
}
