use std::thread::{self, sleep};

use anyhow::Error;
use serde::Deserialize;
use warp::Filter;


#[derive(Deserialize)]
struct Token {
    token: String,
}

pub async fn login(handle: &str) -> Result<(), Error> {
    let client = reqwest::Client::new();
    let res = client
        .post("https://rocksky.app/login")
        .json(&serde_json::json!({
            "handle": handle,
            "cli": true
        }))
        .send()
        .await?;

    let redirect =  res.url().to_string();

    if !redirect.contains("authorize") {
        return Err(anyhow::anyhow!("Failed to login, try again"));
    }

    println!("Open this link in your browser to login: \n{}", redirect);
    open::that(redirect)?;

    let cors = warp::cors()
        .allow_any_origin()
        .allow_methods(vec!["POST"])
        .allow_headers(vec!["content-type"]);

    let routes = warp::post()
    .and(warp::path("token"))
    .and(warp::body::json())
    .and_then(|data: Token| async move {
        let mut home = dirs::home_dir().unwrap();
        home.push(".config/rockbox.org/token");
        std::fs::write(home, data.token).unwrap();

        thread::spawn(move || {
          sleep(std::time::Duration::from_secs(2));
          println!("Login successful!");
          std::process::exit(0);
        });

        Ok::<_, warp::Rejection>(warp::reply::json(&serde_json::json!({
            "ok": 1
        })))
    })
    .with(cors);

    warp::serve(routes).run(([127, 0, 0, 1], 6996)).await;

    Ok(())
}
