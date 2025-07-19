use anyhow::Error;

pub async fn whoami() -> Result<(), Error> {
    // read the token from ~/.config/rockbox.org/token
    let mut home = dirs::home_dir().unwrap();
    home.push(".config/rockbox.org/token");
    match std::fs::read_to_string(home) {
        Ok(token) => {
            let client = reqwest::Client::new();
            let res = client
                .get("https://rocksky.app/profile")
                .header("Authorization", format!("Bearer {}", token))
                .send()
                .await?;

            if res.status().as_u16() != 200 {
                println!("You are not logged in. Use `rockbox login <handle>` to login.");
                return Ok(());
            }

            let user = res.json::<serde_json::Value>().await?;
            println!(
                "You are logged in as: @{} ({})",
                user["handle"].as_str().unwrap(),
                user["displayName"].as_str().unwrap()
            );
        }
        Err(_) => {
            println!("You are not logged in. Use `rockbox login <handle>` to login.");
        }
    }

    Ok(())
}
