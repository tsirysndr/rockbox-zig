use crate::types::*;
use anyhow::Error;
use reqwest::Client;
use tracing::{debug, info, warn};

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
        warn!("RB_TYPESENSE_API_KEY is not set.");
        return Ok(());
    }
    let api_key = api_key.unwrap();
    let res = client
        .post(format!("{}/collections", typesense_host))
        .header("X-TYPESENSE-API-KEY", &api_key)
        .json(&schema)
        .send()
        .await?;

    debug!("Create tracks collection response: {}", res.status());

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
        warn!("RB_TYPESENSE_API_KEY is not set.");
        return Ok(());
    }
    let api_key = api_key.unwrap();
    let res = client
        .post(format!("{}/collections", typesense_host))
        .header("X-TYPESENSE-API-KEY", &api_key)
        .json(&schema)
        .send()
        .await?;

    debug!("Create albums collection response: {}", res.status());

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
        warn!("RB_TYPESENSE_API_KEY is not set.");
        return Ok(());
    }
    let api_key = api_key.unwrap();
    let res = client
        .post(format!("{}/collections", typesense_host))
        .header("X-TYPESENSE-API-KEY", &api_key)
        .json(&schema)
        .send()
        .await?;

    debug!("Create artists collection response: {}", res.status());

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
        warn!("RB_TYPESENSE_API_KEY is not set.");
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

    info!("Insert tracks response: {}", res.status());

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
        warn!("RB_TYPESENSE_API_KEY is not set.");
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

    info!("Insert albums response: {}", res.status());

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
        warn!("RB_TYPESENSE_API_KEY is not set.");
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

    info!("Insert artists response: {}", res.status());

    Ok(())
}

pub async fn search_tracks(query: &str) -> Result<Option<TrackResult>, Error> {
    let client = Client::new();

    let typesense_host = format!(
        "http://localhost:{}",
        std::env::var("RB_TYPESENSE_PORT").unwrap_or_else(|_| "8109".to_string())
    );

    let api_key = std::env::var("RB_TYPESENSE_API_KEY");
    if api_key.is_err() {
        warn!("RB_TYPESENSE_API_KEY is not set.");
        return Ok(None);
    }
    let api_key = api_key.unwrap();
    let res = client
        .get(format!(
            "{}/collections/tracks/documents/search",
            typesense_host,
        ))
        .query(&[
            ("q", query),
            ("query_by", "title,artist,album,path"),
            (
                "include_fields",
                "id,path,title,artist,album,album_artist,bitrate,composer,disc_number,filesize,frequency,length,track_number,year,year_string,genre,md5,album_art,artist_id,album_id,genre_id,created_at,updated_at",
            ),
        ])
        .header("X-TYPESENSE-API-KEY", &api_key)
        .send()
        .await?;

    let text = res.text().await?;
    match serde_json::from_str::<TrackResult>(&text) {
        Ok(result) => Ok(Some(result)),
        Err(e) => {
            warn!("Failed to parse Typesense response: {}", e);
            warn!("Response body: {}", text);
            Err(e.into())
        }
    }
}

pub async fn search_albums(query: &str) -> Result<Option<AlbumResult>, Error> {
    let client = Client::new();

    let typesense_host = format!(
        "http://localhost:{}",
        std::env::var("RB_TYPESENSE_PORT").unwrap_or_else(|_| "8109".to_string())
    );

    let api_key = std::env::var("RB_TYPESENSE_API_KEY");
    if api_key.is_err() {
        warn!("RB_TYPESENSE_API_KEY is not set.");
        return Ok(None);
    }
    let api_key = api_key.unwrap();
    let res = client
        .get(format!(
            "{}/collections/albums/documents/search",
            typesense_host,
        ))
        .query(&[
            ("q", query),
            ("query_by", "title,artist,label"),
            (
                "include_fields",
                "id,title,artist,year,year_string,album_art,md5,artist_id,label",
            ),
        ])
        .header("X-TYPESENSE-API-KEY", &api_key)
        .send()
        .await?;

    Ok(Some(res.json::<AlbumResult>().await?))
}

pub async fn create_playlists_collection() -> Result<(), Error> {
    let client = Client::new();
    let schema = serde_json::json!({
        "name": "playlists",
        "fields": [
            {"name": "name", "type": "string", "sort": true},
            {"name": "description", "type": "string", "optional": true},
            {"name": "image", "type": "string", "optional": true},
            {"name": "is_smart", "type": "bool"},
            {"name": "track_count", "type": "int64"},
        ],
        "default_sorting_field": "name"
    });

    let typesense_host = format!(
        "http://localhost:{}",
        std::env::var("RB_TYPESENSE_PORT").unwrap_or_else(|_| "8109".to_string())
    );

    let api_key = std::env::var("RB_TYPESENSE_API_KEY");
    if api_key.is_err() {
        warn!("RB_TYPESENSE_API_KEY is not set.");
        return Ok(());
    }
    let api_key = api_key.unwrap();
    let res = client
        .post(format!("{}/collections", typesense_host))
        .header("X-TYPESENSE-API-KEY", &api_key)
        .json(&schema)
        .send()
        .await?;

    debug!("Create playlists collection response: {}", res.status());

    Ok(())
}

pub async fn insert_playlists(playlists: Vec<Playlist>) -> Result<(), Error> {
    let client = Client::new();

    let jsonl = playlists
        .into_iter()
        .map(|p| serde_json::to_string(&p).unwrap())
        .collect::<Vec<String>>()
        .join("\n");

    let typesense_host = format!(
        "http://localhost:{}",
        std::env::var("RB_TYPESENSE_PORT").unwrap_or_else(|_| "8109".to_string())
    );

    let api_key = std::env::var("RB_TYPESENSE_API_KEY");
    if api_key.is_err() {
        warn!("RB_TYPESENSE_API_KEY is not set.");
        return Ok(());
    }
    let api_key = api_key.unwrap();
    let res = client
        .post(format!(
            "{}/collections/playlists/documents/import?action=upsert",
            typesense_host
        ))
        .header("X-TYPESENSE-API-KEY", &api_key)
        .header("Content-Type", "text/plain")
        .body(jsonl)
        .send()
        .await?;

    info!("Insert playlists response: {}", res.status());

    Ok(())
}

pub async fn delete_playlist(id: &str) -> Result<(), Error> {
    let client = Client::new();

    let typesense_host = format!(
        "http://localhost:{}",
        std::env::var("RB_TYPESENSE_PORT").unwrap_or_else(|_| "8109".to_string())
    );

    let api_key = std::env::var("RB_TYPESENSE_API_KEY");
    if api_key.is_err() {
        warn!("RB_TYPESENSE_API_KEY is not set.");
        return Ok(());
    }
    let api_key = api_key.unwrap();
    let res = client
        .delete(format!(
            "{}/collections/playlists/documents/{}",
            typesense_host, id
        ))
        .header("X-TYPESENSE-API-KEY", &api_key)
        .send()
        .await?;

    debug!("Delete playlist response: {}", res.status());

    Ok(())
}

pub async fn search_playlists(query: &str) -> Result<Option<PlaylistResult>, Error> {
    let client = Client::new();

    let typesense_host = format!(
        "http://localhost:{}",
        std::env::var("RB_TYPESENSE_PORT").unwrap_or_else(|_| "8109".to_string())
    );

    let api_key = std::env::var("RB_TYPESENSE_API_KEY");
    if api_key.is_err() {
        warn!("RB_TYPESENSE_API_KEY is not set.");
        return Ok(None);
    }
    let api_key = api_key.unwrap();
    let res = client
        .get(format!(
            "{}/collections/playlists/documents/search",
            typesense_host,
        ))
        .query(&[
            ("q", query),
            ("query_by", "name,description"),
            (
                "include_fields",
                "id,name,description,image,is_smart,track_count",
            ),
        ])
        .header("X-TYPESENSE-API-KEY", &api_key)
        .send()
        .await?;

    let text = res.text().await?;
    match serde_json::from_str::<PlaylistResult>(&text) {
        Ok(result) => Ok(Some(result)),
        Err(e) => {
            warn!("Failed to parse Typesense playlists response: {}", e);
            warn!("Response body: {}", text);
            Err(e.into())
        }
    }
}

pub async fn search_artists(query: &str) -> Result<Option<ArtistResult>, Error> {
    let client = Client::new();

    let typesense_host = format!(
        "http://localhost:{}",
        std::env::var("RB_TYPESENSE_PORT").unwrap_or_else(|_| "8109".to_string())
    );

    let api_key = std::env::var("RB_TYPESENSE_API_KEY");
    if api_key.is_err() {
        warn!("RB_TYPESENSE_API_KEY is not set.");
        return Ok(None);
    }
    let api_key = api_key.unwrap();
    let res = client
        .get(format!(
            "{}/collections/artists/documents/search",
            typesense_host,
        ))
        .query(&[
            ("q", query),
            ("query_by", "name"),
            ("include_fields", "id,name,bio,image"),
        ])
        .header("X-TYPESENSE-API-KEY", &api_key)
        .send()
        .await?;

    Ok(Some(res.json::<ArtistResult>().await?))
}
