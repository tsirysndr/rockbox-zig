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
            let local_artists = repo::artist::all(pool.clone()).await?;
            let local_artists = local_artists.into_iter().filter(|v| v.image.is_none());
            let local_artists = local_artists.map(|mut artist| {
                if artist.name == "Theory Of A Deadman" {
                    artist.name = "Theory of a Deadman".to_string();
                }
                artist
            });
            let mut artist_map: HashMap<String, Artist> = HashMap::new();
            let names = local_artists
                .clone()
                .map(|artist| artist.name)
                .collect::<Vec<String>>();

            let client = reqwest::Client::new();
            let response = client
                .get(format!(
                    "{}/xrpc/app.rocksky.artist.getArtists",
                    ROCKSKY_API
                ))
                .query(&[("names", names.join(","))])
                .send()
                .await?;
            let text = response.text().await?;
            let response: Artists = serde_json::from_str(&text)?;
            let artists = response.artists;

            for artist in artists.clone() {
                println!("Loading artist: {}", artist.name.bright_green());
                artist_map.insert(artist.name.clone(), artist);
            }

            println!("Loaded {} artists", artists.len());

            for artist in local_artists {
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
