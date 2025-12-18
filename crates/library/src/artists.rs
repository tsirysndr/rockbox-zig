use std::{collections::HashMap, thread};

use anyhow::Error;
use cuid::cuid1;
use owo_colors::OwoColorize;
use serde::Deserialize;
use sqlx::{Pool, Sqlite};

use crate::repo;

#[derive(Debug, Deserialize, Clone)]
pub struct Artist {
    pub id: String,
    pub name: String,
    pub picture: Option<String>,
    pub sha256: String,
    pub uri: Option<String>,
    #[serde(rename = "playCount")]
    pub play_count: u64,
    #[serde(rename = "uniqueListeners")]
    pub unique_listeners: u32,
    #[serde(default)]
    pub genres: Vec<String>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Artists {
    pub artists: Vec<Artist>,
}

const ROCKSKY_API: &str = "https://api.rocksky.app";

pub fn update_metadata(pool: Pool<Sqlite>) -> Result<(), Error> {
    thread::spawn(move || {
        let runtime = tokio::runtime::Runtime::new().unwrap();
        let result = runtime.block_on(async {
            let mut offset = 0;
            let limit = 500;
            let mut artist_map: HashMap<String, Artist> = HashMap::new();

            loop {
                let client = reqwest::Client::new();
                let response = client
                    .get(format!(
                        "{}/xrpc/app.rocksky.artist.getArtists",
                        ROCKSKY_API
                    ))
                    .query(&[("limit", limit), ("offset", offset)])
                    .send()
                    .await?;
                let text = response.text().await?;
                let response: Artists = serde_json::from_str(&text)?;
                let artists = response.artists;

                for artist in artists.clone() {
                    println!("Loading artist: {}", artist.name.bright_green());
                    artist_map.insert(artist.name.clone(), artist);
                }

                if artists.is_empty() {
                    break;
                }

                offset += limit;
                println!("Loaded {} artists", offset);
            }

            let artists = repo::artist::all(pool.clone()).await?;
            let artists = artists.into_iter().filter(|v| v.image.is_none());

            for artist in artists {
                println!("Updating artist: {}", artist.name.bright_green());
                let artist_id = artist.id;
                if let Some(artist) = artist_map.get(&artist.name) {
                    repo::artist::update_genres(&pool, &artist_id, &artist.genres.join(", "))
                        .await?;
                    if let Some(picture) = artist.picture.clone() {
                        repo::artist::update_picture(&pool, &artist_id, &picture).await?;
                    }
                    for genre in &artist.genres {
                        println!("Saving genre: {}", genre.bright_green());
                        let id = cuid1()?;
                        repo::genre::save(&pool, &id, genre).await?;
                        repo::artist::save_artist_genre(&pool, &cuid1()?, &artist_id, &id).await?;
                    }
                }
            }

            Ok::<(), Error>(())
        });

        match result {
            Ok(_) => {}
            Err(e) => eprintln!("Error updating artists: {}", e),
        }
    });
    Ok(())
}
