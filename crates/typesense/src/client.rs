use anyhow::Error;
use reqwest::Client;
use rockbox_library::entity;
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Track {
    pub id: String,
    pub path: String,
    pub title: String,
    pub artist: String,
    pub album: String,
    pub album_artist: String,
    pub bitrate: i64,
    pub composer: String,
    pub disc_number: i64,
    pub filesize: i64,
    pub frequency: i64,
    pub length: i64,
    pub track_number: i64,
    pub year: i32,
    pub year_string: String,
    pub genre: String,
    pub md5: String,
    pub album_art: Option<String>,
    pub artist_id: Option<String>,
    pub album_id: Option<String>,
    pub genre_id: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

impl From<entity::track::Track> for Track {
    fn from(track: entity::track::Track) -> Self {
        Self {
            id: track.id,
            path: track.path,
            title: track.title,
            artist: track.artist,
            album: track.album,
            album_artist: track.album_artist,
            bitrate: track.bitrate as i64,
            composer: track.composer,
            disc_number: track.disc_number as i64,
            filesize: track.filesize as i64,
            frequency: track.frequency as i64,
            length: track.length as i64,
            track_number: track.track_number.unwrap_or_default() as i64,
            year: track.year.unwrap_or_default() as i32,
            year_string: track.year_string.unwrap_or_default(),
            genre: track.genre.unwrap_or_default(),
            md5: track.md5,
            album_art: track.album_art,
            artist_id: Some(track.artist_id),
            album_id: Some(track.album_id),
            genre_id: Some(track.genre_id),
            created_at: track.created_at.to_rfc3339(),
            updated_at: track.updated_at.to_rfc3339(),
        }
    }
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Album {
    pub id: String,
    pub title: String,
    pub artist: String,
    pub year: i32,
    pub year_string: String,
    pub album_art: Option<String>,
    pub md5: String,
    pub artist_id: String,
    pub label: Option<String>,
}

impl From<entity::album::Album> for Album {
    fn from(album: entity::album::Album) -> Self {
        Self {
            id: album.id,
            title: album.title,
            artist: album.artist,
            year: album.year as i32,
            year_string: album.year_string,
            album_art: album.album_art,
            md5: album.md5,
            artist_id: album.artist_id,
            label: album.label,
        }
    }
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Artist {
    pub id: String,
    pub name: String,
    pub bio: Option<String>,
    pub image: Option<String>,
}

impl From<entity::artist::Artist> for Artist {
    fn from(artist: entity::artist::Artist) -> Self {
        Self {
            id: artist.id,
            name: artist.name,
            bio: artist.bio,
            image: artist.image,
        }
    }
}

pub async fn create_tracks_collection() -> Result<(), Error> {
    let client = Client::new();
    let schema = serde_json::json!({
        "name": "tracks",
        "fields": [
            {"name": "path", "type": "string"},
            {"name": "title", "type": "string", "sort": true},
            {"name": "artist", "type": "string"},
            {"name": "album", "type": "string"},
            {"name": "album_artist", "type": "string"},
            {"name": "bitrate", "type": "int32"},
            {"name": "composer", "type": "string"},
            {"name": "disc_number", "type": "int32"},
            {"name": "filesize", "type": "int64"},
            {"name": "frequency", "type": "int32"},
            {"name": "length", "type": "int64"},
            {"name": "track_number", "type": "int32"},
            {"name": "year", "type": "int32"},
            {"name": "year_string", "type": "string"},
            {"name": "genre", "type": "string"},
            {"name": "md5", "type": "string"},
            {"name": "album_art", "type": "string", "optional": true},
            {"name": "artist_id", "type": "string", "optional": true},
            {"name": "album_id", "type": "string", "optional": true},
            {"name": "genre_id", "type": "string", "optional": true}
        ],
        "default_sorting_field": "title"
    });

    let typesense_host = format!(
        "http://localhost:{}",
        std::env::var("RB_TYPESENSE_PORT").unwrap_or_else(|_| "8109".to_string())
    );

    let api_key = std::env::var("RB_TYPESENSE_API_KEY");
    if api_key.is_err() {
        println!("Warning: RB_TYPESENSE_API_KEY is not set.");
        return Ok(());
    }
    let api_key = api_key.unwrap();
    let res = client
        .post(format!("{}/collections", typesense_host))
        .header("X-TYPESENSE-API-KEY", &api_key)
        .json(&schema)
        .send()
        .await?;

    println!("Create tracks collection response: {}", res.status());

    Ok(())
}

pub async fn create_albums_collection() -> Result<(), Error> {
    let client = Client::new();
    let schema = serde_json::json!({
        "name": "albums",
        "fields": [
            {"name": "title", "type": "string", "sort": true},
            {"name": "artist", "type": "string"},
            {"name": "year", "type": "int32"},
            {"name": "year_string", "type": "string"},
            {"name": "album_art", "type": "string", "optional": true},
            {"name": "md5", "type": "string"},
            {"name": "artist_id", "type": "string"},
            {"name": "label", "type": "string", "optional": true},
        ],
        "default_sorting_field": "title"
    });

    let typesense_host = format!(
        "http://localhost:{}",
        std::env::var("RB_TYPESENSE_PORT").unwrap_or_else(|_| "8109".to_string())
    );

    let api_key = std::env::var("RB_TYPESENSE_API_KEY");
    if api_key.is_err() {
        println!("Warning: RB_TYPESENSE_API_KEY is not set.");
        return Ok(());
    }
    let api_key = api_key.unwrap();
    let res = client
        .post(format!("{}/collections", typesense_host))
        .header("X-TYPESENSE-API-KEY", &api_key)
        .json(&schema)
        .send()
        .await?;

    println!("Create albums collection response: {}", res.status());

    Ok(())
}

pub async fn create_artists_collection() -> Result<(), Error> {
    let client = Client::new();
    let schema = serde_json::json!({
        "name": "artists",
        "fields": [
            {"name": "name", "type": "string", "sort": true},
            {"name": "bio", "type": "string", "optional": true},
            {"name": "image", "type": "string", "optional": true},
        ],
        "default_sorting_field": "name"
    });

    let typesense_host = format!(
        "http://localhost:{}",
        std::env::var("RB_TYPESENSE_PORT").unwrap_or_else(|_| "8109".to_string())
    );

    let api_key = std::env::var("RB_TYPESENSE_API_KEY");
    if api_key.is_err() {
        println!("Warning: RB_TYPESENSE_API_KEY is not set.");
        return Ok(());
    }
    let api_key = api_key.unwrap();
    let res = client
        .post(format!("{}/collections", typesense_host))
        .header("X-TYPESENSE-API-KEY", &api_key)
        .json(&schema)
        .send()
        .await?;

    println!("Create artists collection response: {}", res.status());

    Ok(())
}

pub async fn insert_tracks(tracks: Vec<Track>) -> Result<(), Error> {
    let client = Client::new();

    let jsonl = tracks
        .into_iter()
        .map(|track| serde_json::to_string(&track).unwrap())
        .collect::<Vec<String>>()
        .join("\n");

    let typesense_host = format!(
        "http://localhost:{}",
        std::env::var("RB_TYPESENSE_PORT").unwrap_or_else(|_| "8109".to_string())
    );

    let api_key = std::env::var("RB_TYPESENSE_API_KEY");
    if api_key.is_err() {
        println!("Warning: RB_TYPESENSE_API_KEY is not set.");
        return Ok(());
    }
    let api_key = api_key.unwrap();
    let res = client
        .post(format!(
            "{}/collections/tracks/documents/import?action=upsert",
            typesense_host
        ))
        .header("X-TYPESENSE-API-KEY", &api_key)
        .header("Content-Type", "text/plain")
        .body(jsonl)
        .send()
        .await?;

    println!("Insert tracks response: {}", res.status());

    Ok(())
}

pub async fn insert_albums(albums: Vec<Album>) -> Result<(), Error> {
    let client = Client::new();

    let jsonl = albums
        .into_iter()
        .map(|album| serde_json::to_string(&album).unwrap())
        .collect::<Vec<String>>()
        .join("\n");

    let typesense_host = format!(
        "http://localhost:{}",
        std::env::var("RB_TYPESENSE_PORT").unwrap_or_else(|_| "8109".to_string())
    );

    let api_key = std::env::var("RB_TYPESENSE_API_KEY");
    if api_key.is_err() {
        println!("Warning: RB_TYPESENSE_API_KEY is not set.");
        return Ok(());
    }
    let api_key = api_key.unwrap();
    let res = client
        .post(format!(
            "{}/collections/albums/documents/import?action=upsert",
            typesense_host
        ))
        .header("X-TYPESENSE-API-KEY", &api_key)
        .header("Content-Type", "text/plain")
        .body(jsonl)
        .send()
        .await?;

    println!("Insert albums response: {}", res.status());

    Ok(())
}

pub async fn insert_artists(artists: Vec<Artist>) -> Result<(), Error> {
    let client = Client::new();

    let jsonl = artists
        .into_iter()
        .map(|artist| serde_json::to_string(&artist).unwrap())
        .collect::<Vec<String>>()
        .join("\n");

    let typesense_host = format!(
        "http://localhost:{}",
        std::env::var("RB_TYPESENSE_PORT").unwrap_or_else(|_| "8109".to_string())
    );

    let api_key = std::env::var("RB_TYPESENSE_API_KEY");
    if api_key.is_err() {
        println!("Warning: RB_TYPESENSE_API_KEY is not set.");
        return Ok(());
    }
    let api_key = api_key.unwrap();
    let res = client
        .post(format!(
            "{}/collections/artists/documents/import?action=upsert",
            typesense_host
        ))
        .header("X-TYPESENSE-API-KEY", &api_key)
        .header("Content-Type", "text/plain")
        .body(jsonl)
        .send()
        .await?;

    println!("Insert artists response: {}", res.status());

    Ok(())
}
