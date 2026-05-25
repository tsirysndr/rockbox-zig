use actix_web::{web, HttpRequest, HttpResponse};
use rand::seq::SliceRandom;
use rockbox_library::{audio_scan::scan_audio_files, entity::favourites::Favourites, repo};
use serde::Deserialize;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::sync::atomic::Ordering;
use uuid::Uuid;

use super::{response, SubsonicState};

// ── Common query params ───────────────────────────────────────────────────────

#[derive(Deserialize, Default)]
pub struct CommonParams {
    pub u: Option<String>,
    pub p: Option<String>,
    pub t: Option<String>,
    pub s: Option<String>,
    pub f: Option<String>,
}

#[derive(Deserialize, Default)]
pub struct IdParam {
    pub u: Option<String>,
    pub p: Option<String>,
    pub t: Option<String>,
    pub s: Option<String>,
    pub f: Option<String>,
    pub id: Option<String>,
}

#[derive(Deserialize, Default)]
pub struct SearchParams {
    pub u: Option<String>,
    pub p: Option<String>,
    pub t: Option<String>,
    pub s: Option<String>,
    pub f: Option<String>,
    pub query: Option<String>,
    #[serde(rename = "artistCount")]
    pub artist_count: Option<i64>,
    #[serde(rename = "albumCount")]
    pub album_count: Option<i64>,
    #[serde(rename = "songCount")]
    pub song_count: Option<i64>,
    #[serde(rename = "artistOffset")]
    pub artist_offset: Option<i64>,
    #[serde(rename = "albumOffset")]
    pub album_offset: Option<i64>,
    #[serde(rename = "songOffset")]
    pub song_offset: Option<i64>,
}

#[derive(Deserialize, Default)]
pub struct AlbumListParams {
    pub u: Option<String>,
    pub p: Option<String>,
    pub t: Option<String>,
    pub s: Option<String>,
    pub f: Option<String>,
    #[serde(rename = "type")]
    pub list_type: Option<String>,
    pub size: Option<i64>,
    pub offset: Option<i64>,
    #[serde(rename = "fromYear")]
    pub from_year: Option<i64>,
    #[serde(rename = "toYear")]
    pub to_year: Option<i64>,
}

#[derive(Deserialize, Default)]
pub struct ArtistInfoParams {
    pub u: Option<String>,
    pub p: Option<String>,
    pub t: Option<String>,
    pub s: Option<String>,
    pub f: Option<String>,
    pub id: Option<String>,
    pub count: Option<i64>,
}

#[derive(Deserialize, Default)]
pub struct TopSongsParams {
    pub u: Option<String>,
    pub p: Option<String>,
    pub t: Option<String>,
    pub s: Option<String>,
    pub f: Option<String>,
    pub artist: Option<String>,
    pub count: Option<i64>,
}

#[derive(Deserialize, Default)]
pub struct LyricsParams {
    pub u: Option<String>,
    pub p: Option<String>,
    pub t: Option<String>,
    pub s: Option<String>,
    pub f: Option<String>,
    pub id: Option<String>,
    pub artist: Option<String>,
    pub title: Option<String>,
}

#[derive(Deserialize, Default)]
pub struct RandomSongsParams {
    pub u: Option<String>,
    pub p: Option<String>,
    pub t: Option<String>,
    pub s: Option<String>,
    pub f: Option<String>,
    pub size: Option<i64>,
    #[serde(rename = "fromYear")]
    pub from_year: Option<i64>,
    #[serde(rename = "toYear")]
    pub to_year: Option<i64>,
    pub genre: Option<String>,
}

#[derive(Deserialize, Default)]
pub struct ScrobbleParams {
    pub u: Option<String>,
    pub p: Option<String>,
    pub t: Option<String>,
    pub s: Option<String>,
    pub f: Option<String>,
    pub id: Option<String>,
    pub submission: Option<bool>,
}

#[derive(Deserialize, Default)]
pub struct StarParams {
    pub u: Option<String>,
    pub p: Option<String>,
    pub t: Option<String>,
    pub s: Option<String>,
    pub f: Option<String>,
    pub id: Option<String>,
    #[serde(rename = "albumId")]
    pub album_id: Option<String>,
    #[serde(rename = "artistId")]
    pub artist_id: Option<String>,
}

#[derive(Deserialize, Default)]
pub struct SongsByGenreParams {
    pub u: Option<String>,
    pub p: Option<String>,
    pub t: Option<String>,
    pub s: Option<String>,
    pub f: Option<String>,
    pub genre: Option<String>,
    pub count: Option<i64>,
    pub offset: Option<i64>,
}

// ── Auth helper ───────────────────────────────────────────────────────────────

fn auth_check(
    state: &SubsonicState,
    u: Option<&str>,
    p: Option<&str>,
    t: Option<&str>,
    s: Option<&str>,
    f: Option<&str>,
) -> Option<HttpResponse> {
    if !super::check_auth(&state.username, &state.password, u, p, t, s) {
        return Some(response::respond_error(f, 40, "Wrong username or password"));
    }
    None
}

// ── Mappers ───────────────────────────────────────────────────────────────────

fn track_to_child(t: &rockbox_library::entity::track::Track) -> Value {
    let content_type = mime_for_path(&t.path);
    let suffix = t.path.rsplit('.').next().unwrap_or("").to_lowercase();
    json!({
        "id": t.id,
        "parent": t.album_id,
        "isDir": false,
        "title": t.title,
        "album": t.album,
        "artist": t.artist,
        "track": t.track_number,
        "year": t.year,
        "genre": t.genre,
        "coverArt": format!("al-{}", t.album_id),
        "size": t.filesize,
        "contentType": content_type,
        "suffix": suffix,
        "duration": t.length / 1000,
        "bitRate": t.bitrate,
        "path": t.path,
        "isVideo": false,
        "discNumber": t.disc_number,
        "albumId": if t.album_id.is_empty() { Value::Null } else { json!(t.album_id) },
        "artistId": if t.artist_id.is_empty() { Value::Null } else { json!(t.artist_id) },
        "type": "music",
    })
}

fn album_to_child(a: &rockbox_library::entity::album::Album, song_count: i64) -> Value {
    json!({
        "id": a.id,
        "name": a.title,
        "title": a.title,
        "artist": a.artist,
        "artistId": a.artist_id,
        "songCount": song_count,
        "duration": 0,
        "year": if a.year > 0 { json!(a.year) } else { Value::Null },
        "coverArt": format!("al-{}", a.id),
        "created": "2020-01-01T00:00:00Z",
    })
}

fn artist_to_json(a: &rockbox_library::entity::artist::Artist, album_count: i64) -> Value {
    json!({
        "id": a.id,
        "name": a.name,
        "albumCount": album_count,
        "coverArt": format!("ar-{}", a.id),
    })
}

fn mime_for_path(path: &str) -> &'static str {
    match path.rsplit('.').next().map(|s| s.to_lowercase()).as_deref() {
        Some("mp3") => "audio/mpeg",
        Some("flac") => "audio/flac",
        Some("ogg") => "audio/ogg",
        Some("m4a") | Some("aac") | Some("mp4") => "audio/mp4",
        Some("wav") => "audio/wav",
        Some("wma") => "audio/x-ms-wma",
        Some("opus") => "audio/opus",
        Some("aiff") | Some("aif") => "audio/aiff",
        Some("wv") => "audio/x-wavpack",
        Some("mpc") => "audio/x-musepack",
        Some("ape") => "audio/x-ape",
        _ => "audio/mpeg",
    }
}

/// Build artist index (alphabetical grouping A-Z + #).
/// `album_counts` maps artist_id → number of albums.
fn build_artist_index(
    artists: &[rockbox_library::entity::artist::Artist],
    album_counts: &HashMap<String, usize>,
) -> Vec<Value> {
    let mut groups: HashMap<String, Vec<Value>> = HashMap::new();
    for a in artists {
        let first = a
            .name
            .chars()
            .next()
            .map(|c| c.to_uppercase().next().unwrap_or(c))
            .unwrap_or('#');
        let key = if first.is_alphabetic() {
            first.to_string()
        } else {
            "#".to_string()
        };
        let count = album_counts.get(&a.id).copied().unwrap_or(0);
        groups.entry(key).or_default().push(json!({
            "id": a.id,
            "name": a.name,
            "albumCount": count,
            "coverArt": format!("ar-{}", a.id),
        }));
    }
    let mut keys: Vec<String> = groups.keys().cloned().collect();
    keys.sort();
    keys.into_iter()
        .map(|k| json!({ "name": k, "artist": groups[&k] }))
        .collect()
}

// ── Handlers ─────────────────────────────────────────────────────────────────

pub async fn ping(
    state: web::Data<SubsonicState>,
    query: web::Query<CommonParams>,
) -> HttpResponse {
    let q = query.into_inner();
    let f = q.f.as_deref();
    if let Some(r) = auth_check(
        &state,
        q.u.as_deref(),
        q.p.as_deref(),
        q.t.as_deref(),
        q.s.as_deref(),
        f,
    ) {
        return r;
    }
    response::respond(f, json!({}), "")
}

pub async fn get_music_folders(
    state: web::Data<SubsonicState>,
    query: web::Query<CommonParams>,
) -> HttpResponse {
    let q = query.into_inner();
    let f = q.f.as_deref();
    if let Some(r) = auth_check(
        &state,
        q.u.as_deref(),
        q.p.as_deref(),
        q.t.as_deref(),
        q.s.as_deref(),
        f,
    ) {
        return r;
    }
    let music_dir = std::env::var("ROCKBOX_LIBRARY").unwrap_or_else(|_| {
        let home = std::env::var("HOME").unwrap_or_default();
        format!("{}/Music", home)
    });
    let json_data = json!({
        "musicFolders": {
            "musicFolder": [{"id": 1, "name": music_dir}]
        }
    });
    let xml = format!(
        r#"<musicFolders><musicFolder id="1" name="{}"/></musicFolders>"#,
        xml_escape(&music_dir)
    );
    response::respond(f, json_data, &xml)
}

pub async fn get_artists(
    state: web::Data<SubsonicState>,
    query: web::Query<CommonParams>,
) -> HttpResponse {
    let q = query.into_inner();
    let f = q.f.as_deref();
    if let Some(r) = auth_check(
        &state,
        q.u.as_deref(),
        q.p.as_deref(),
        q.t.as_deref(),
        q.s.as_deref(),
        f,
    ) {
        return r;
    }
    let artists = match repo::artist::all(state.pool.clone()).await {
        Ok(a) => a,
        Err(e) => {
            tracing::error!("getArtists: {e}");
            return response::respond_error(f, 0, "database error");
        }
    };
    let album_counts = repo::album::count_by_artist(state.pool.clone())
        .await
        .unwrap_or_default();
    let index = build_artist_index(&artists, &album_counts);
    let json_data = json!({
        "artists": {
            "ignoredArticles": "The An A Die Das Ein",
            "index": index,
        }
    });
    let xml_entries: String = index
        .iter()
        .map(|g| {
            let name = g["name"].as_str().unwrap_or("");
            let inner: String = g["artist"]
                .as_array()
                .map(|arr| {
                    arr.iter()
                        .map(|a| {
                            format!(
                                r#"<artist id="{}" name="{}"/>"#,
                                xml_escape(a["id"].as_str().unwrap_or("")),
                                xml_escape(a["name"].as_str().unwrap_or(""))
                            )
                        })
                        .collect()
                })
                .unwrap_or_default();
            format!(r#"<index name="{}">{}</index>"#, xml_escape(name), inner)
        })
        .collect();
    let xml = format!(r#"<artists ignoredArticles="The An A Die Das Ein">{xml_entries}</artists>"#);
    response::respond(f, json_data, &xml)
}

pub async fn get_artist(
    state: web::Data<SubsonicState>,
    query: web::Query<IdParam>,
) -> HttpResponse {
    let q = query.into_inner();
    let f = q.f.as_deref();
    if let Some(r) = auth_check(
        &state,
        q.u.as_deref(),
        q.p.as_deref(),
        q.t.as_deref(),
        q.s.as_deref(),
        f,
    ) {
        return r;
    }
    let id = match q.id.as_deref() {
        Some(id) => id,
        None => return response::respond_error(f, 10, "Required parameter is missing: id"),
    };
    let artist = match repo::artist::find(state.pool.clone(), id).await {
        Ok(Some(a)) => a,
        Ok(None) => return response::respond_error(f, 70, "Artist not found"),
        Err(e) => {
            tracing::error!("getArtist: {e}");
            return response::respond_error(f, 0, "database error");
        }
    };
    let albums = match repo::album::find_by_artist(state.pool.clone(), id).await {
        Ok(a) => a,
        Err(e) => {
            tracing::error!("getArtist albums: {e}");
            vec![]
        }
    };
    let album_jsons: Vec<Value> = albums.iter().map(|a| album_to_child(a, 0)).collect();
    let json_data = json!({
        "artist": {
            "id": artist.id,
            "name": artist.name,
            "albumCount": album_jsons.len(),
            "coverArt": format!("ar-{}", artist.id),
            "album": album_jsons,
        }
    });
    let albums_xml: String = album_jsons.iter().map(|a| album_elem_xml(a)).collect();
    let xml = format!(
        r#"<artist id="{}" name="{}" albumCount="{}">{albums_xml}</artist>"#,
        xml_escape(&artist.id),
        xml_escape(&artist.name),
        album_jsons.len()
    );
    response::respond(f, json_data, &xml)
}

pub async fn get_album(
    state: web::Data<SubsonicState>,
    query: web::Query<IdParam>,
) -> HttpResponse {
    let q = query.into_inner();
    let f = q.f.as_deref();
    if let Some(r) = auth_check(
        &state,
        q.u.as_deref(),
        q.p.as_deref(),
        q.t.as_deref(),
        q.s.as_deref(),
        f,
    ) {
        return r;
    }
    let id = match q.id.as_deref() {
        Some(id) => id,
        None => return response::respond_error(f, 10, "Required parameter is missing: id"),
    };
    let album = match repo::album::find(state.pool.clone(), id).await {
        Ok(Some(a)) => a,
        Ok(None) => return response::respond_error(f, 70, "Album not found"),
        Err(e) => {
            tracing::error!("getAlbum: {e}");
            return response::respond_error(f, 0, "database error");
        }
    };
    let tracks = match repo::album_tracks::find_by_album(state.pool.clone(), id).await {
        Ok(t) => t,
        Err(e) => {
            tracing::error!("getAlbum tracks: {e}");
            vec![]
        }
    };
    let song_jsons: Vec<Value> = tracks.iter().map(track_to_child).collect();
    let json_data = json!({
        "album": {
            "id": album.id,
            "name": album.title,
            "artist": album.artist,
            "artistId": album.artist_id,
            "coverArt": format!("al-{}", album.id),
            "songCount": song_jsons.len(),
            "duration": tracks.iter().map(|t| t.length / 1000).sum::<u32>(),
            "year": if album.year > 0 { json!(album.year) } else { Value::Null },
            "song": song_jsons,
        }
    });
    let songs_xml: String = song_jsons.iter().map(|s| song_elem_xml(s)).collect();
    let xml = format!(
        r#"<album id="{}" name="{}" artist="{}" artistId="{}" songCount="{}">{songs_xml}</album>"#,
        xml_escape(&album.id),
        xml_escape(&album.title),
        xml_escape(&album.artist),
        xml_escape(&album.artist_id),
        song_jsons.len()
    );
    response::respond(f, json_data, &xml)
}

pub async fn get_song(state: web::Data<SubsonicState>, query: web::Query<IdParam>) -> HttpResponse {
    let q = query.into_inner();
    let f = q.f.as_deref();
    if let Some(r) = auth_check(
        &state,
        q.u.as_deref(),
        q.p.as_deref(),
        q.t.as_deref(),
        q.s.as_deref(),
        f,
    ) {
        return r;
    }
    let id = match q.id.as_deref() {
        Some(id) => id,
        None => return response::respond_error(f, 10, "Required parameter is missing: id"),
    };
    let track = match repo::track::find(state.pool.clone(), id).await {
        Ok(Some(t)) => t,
        Ok(None) => return response::respond_error(f, 70, "Song not found"),
        Err(e) => {
            tracing::error!("getSong: {e}");
            return response::respond_error(f, 0, "database error");
        }
    };
    let song = track_to_child(&track);
    let xml = song_elem_xml(&song);
    response::respond(f, json!({"song": song}), &xml)
}

pub async fn stream(
    state: web::Data<SubsonicState>,
    query: web::Query<IdParam>,
    req: HttpRequest,
) -> HttpResponse {
    let q = query.into_inner();
    let f = q.f.as_deref();
    if let Some(r) = auth_check(
        &state,
        q.u.as_deref(),
        q.p.as_deref(),
        q.t.as_deref(),
        q.s.as_deref(),
        f,
    ) {
        return r;
    }
    let id = match q.id.as_deref() {
        Some(id) => id,
        None => return response::respond_error(f, 10, "Required parameter is missing: id"),
    };
    let track = match repo::track::find(state.pool.clone(), id).await {
        Ok(Some(t)) => t,
        Ok(None) => return response::respond_error(f, 70, "Song not found"),
        Err(e) => {
            tracing::error!("stream: {e}");
            return response::respond_error(f, 0, "database error");
        }
    };
    let content_type = mime_for_path(&track.path);
    let file_size = match std::fs::metadata(&track.path) {
        Ok(m) => m.len(),
        Err(e) => {
            tracing::error!("stream stat {}: {e}", track.path);
            return response::respond_error(f, 0, "could not read file");
        }
    };

    // Handle Range requests for seeking support.
    if let Some(range_hdr) = req.headers().get(actix_web::http::header::RANGE) {
        if let Ok(range_str) = range_hdr.to_str() {
            if let Some(range) = range_str.strip_prefix("bytes=") {
                let mut parts = range.splitn(2, '-');
                let start: u64 = parts.next().and_then(|s| s.parse().ok()).unwrap_or(0);
                let end: u64 = parts
                    .next()
                    .filter(|s| !s.is_empty())
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(file_size.saturating_sub(1))
                    .min(file_size.saturating_sub(1));
                if start <= end {
                    use std::io::{Read, Seek, SeekFrom};
                    match std::fs::File::open(&track.path) {
                        Ok(mut file) => {
                            let _ = file.seek(SeekFrom::Start(start));
                            let length = (end - start + 1) as usize;
                            let mut buf = vec![0u8; length];
                            let n = file.read(&mut buf).unwrap_or(0);
                            buf.truncate(n);
                            let actual_end = start + n as u64 - 1;
                            return HttpResponse::PartialContent()
                                .content_type(content_type)
                                .insert_header(("Accept-Ranges", "bytes"))
                                .insert_header(("Content-Length", n.to_string()))
                                .insert_header((
                                    "Content-Range",
                                    format!("bytes {}-{}/{}", start, actual_end, file_size),
                                ))
                                .body(buf);
                        }
                        Err(e) => {
                            tracing::error!("stream range open {}: {e}", track.path);
                            return response::respond_error(f, 0, "could not read file");
                        }
                    }
                }
            }
        }
    }

    match std::fs::read(&track.path) {
        Ok(data) => HttpResponse::Ok()
            .content_type(content_type)
            .insert_header(("Accept-Ranges", "bytes"))
            .insert_header(("Content-Length", file_size.to_string()))
            .insert_header((
                "Content-Disposition",
                format!("attachment; filename=\"{}\"", safe_filename(&track.path)),
            ))
            .body(data),
        Err(e) => {
            tracing::error!("stream read {}: {e}", track.path);
            response::respond_error(f, 0, "could not read file")
        }
    }
}

pub async fn get_cover_art(
    state: web::Data<SubsonicState>,
    query: web::Query<IdParam>,
    _req: HttpRequest,
) -> HttpResponse {
    let q = query.into_inner();
    let f = q.f.as_deref();
    if let Some(r) = auth_check(
        &state,
        q.u.as_deref(),
        q.p.as_deref(),
        q.t.as_deref(),
        q.s.as_deref(),
        f,
    ) {
        return r;
    }
    let id = match q.id.as_deref() {
        Some(id) => id,
        None => return HttpResponse::NotFound().finish(),
    };

    // id may be "al-<albumId>", "ar-<artistId>", or a track id.
    let art_path = if let Some(album_id) = id.strip_prefix("al-") {
        cover_art_for_album(&state, album_id).await
    } else if let Some(artist_id) = id.strip_prefix("ar-") {
        cover_art_for_artist(&state, artist_id).await
    } else {
        // Treat as track id
        repo::track::find(state.pool.clone(), id)
            .await
            .ok()
            .flatten()
            .and_then(|t| t.album_art)
    };

    match art_path {
        None => HttpResponse::NotFound().finish(),
        Some(art) => serve_art(&art).await,
    }
}

/// Resolve a stored art value to bytes and serve it.
/// Art is either:
///   - a bare filename like "abc123.jpg" → stored under ~/.config/rockbox.org/covers/
///   - an absolute path starting with '/'
///   - an HTTP/HTTPS URL (artist images from Rocksky)
async fn serve_art(art: &str) -> HttpResponse {
    if art.starts_with("http://") || art.starts_with("https://") {
        // Proxy remote image (artist images from Rocksky).
        match reqwest::get(art).await {
            Ok(resp) => {
                let mime = resp
                    .headers()
                    .get("content-type")
                    .and_then(|v| v.to_str().ok())
                    .unwrap_or("image/jpeg")
                    .to_string();
                match resp.bytes().await {
                    Ok(data) => HttpResponse::Ok().content_type(mime).body(data.to_vec()),
                    Err(_) => HttpResponse::NotFound().finish(),
                }
            }
            Err(e) => {
                tracing::warn!("getCoverArt proxy {art}: {e}");
                HttpResponse::NotFound().finish()
            }
        }
    } else {
        // Local file — resolve relative filenames to the covers directory.
        let full_path = if art.starts_with('/') {
            art.to_string()
        } else {
            let home = std::env::var("HOME").unwrap_or_default();
            format!("{}/.config/rockbox.org/covers/{}", home, art)
        };
        match std::fs::read(&full_path) {
            Ok(data) => {
                let mime = mime_guess::from_path(&full_path)
                    .first_or_octet_stream()
                    .to_string();
                HttpResponse::Ok().content_type(mime).body(data)
            }
            Err(e) => {
                tracing::warn!("getCoverArt read {full_path}: {e}");
                HttpResponse::NotFound().finish()
            }
        }
    }
}

async fn cover_art_for_album(state: &SubsonicState, album_id: &str) -> Option<String> {
    // Prefer the album's own art field (populated by the scanner).
    if let Ok(Some(album)) = repo::album::find(state.pool.clone(), album_id).await {
        if album.album_art.is_some() {
            return album.album_art;
        }
    }
    // Fall back to art from any track in the album.
    repo::album_tracks::find_by_album(state.pool.clone(), album_id)
        .await
        .ok()?
        .into_iter()
        .find_map(|t| t.album_art)
}

async fn cover_art_for_artist(state: &SubsonicState, artist_id: &str) -> Option<String> {
    let artist = repo::artist::find(state.pool.clone(), artist_id)
        .await
        .ok()??;
    if artist.image.is_some() {
        return artist.image;
    }
    // Fall back to cover art from a track on one of the artist's albums
    let albums = repo::album::find_by_artist(state.pool.clone(), artist_id)
        .await
        .ok()?;
    for album in albums {
        if let Ok(tracks) = repo::album_tracks::find_by_album(state.pool.clone(), &album.id).await {
            if let Some(art) = tracks.into_iter().find_map(|t| t.album_art) {
                return Some(art);
            }
        }
    }
    None
}

pub async fn search3(
    state: web::Data<SubsonicState>,
    query: web::Query<SearchParams>,
) -> HttpResponse {
    let q = query.into_inner();
    let f = q.f.as_deref();
    if let Some(r) = auth_check(
        &state,
        q.u.as_deref(),
        q.p.as_deref(),
        q.t.as_deref(),
        q.s.as_deref(),
        f,
    ) {
        return r;
    }
    let term = q.query.as_deref().unwrap_or("").to_lowercase();
    let artist_limit = q.artist_count.unwrap_or(20) as usize;
    let album_limit = q.album_count.unwrap_or(20) as usize;
    let song_limit = q.song_count.unwrap_or(20) as usize;
    let artist_offset = q.artist_offset.unwrap_or(0) as usize;
    let album_offset = q.album_offset.unwrap_or(0) as usize;
    let song_offset = q.song_offset.unwrap_or(0) as usize;

    let all_artists = repo::artist::all(state.pool.clone())
        .await
        .unwrap_or_default();
    let artists: Vec<Value> = all_artists
        .iter()
        .filter(|a| term.is_empty() || a.name.to_lowercase().contains(&term))
        .skip(artist_offset)
        .take(artist_limit)
        .map(|a| artist_to_json(a, 0))
        .collect();

    let all_albums = repo::album::all(state.pool.clone())
        .await
        .unwrap_or_default();
    let albums: Vec<Value> = all_albums
        .iter()
        .filter(|a| {
            term.is_empty()
                || a.title.to_lowercase().contains(&term)
                || a.artist.to_lowercase().contains(&term)
        })
        .skip(album_offset)
        .take(album_limit)
        .map(|a| album_to_child(a, 0))
        .collect();

    let all_tracks = repo::track::all(state.pool.clone())
        .await
        .unwrap_or_default();
    let songs: Vec<Value> = all_tracks
        .iter()
        .filter(|t| {
            term.is_empty()
                || t.title.to_lowercase().contains(&term)
                || t.artist.to_lowercase().contains(&term)
                || t.album.to_lowercase().contains(&term)
        })
        .skip(song_offset)
        .take(song_limit)
        .map(track_to_child)
        .collect();

    let json_data = json!({
        "searchResult3": {
            "artist": artists,
            "album": albums,
            "song": songs,
        }
    });
    let artists_xml: String = artists
        .iter()
        .map(|a| {
            format!(
                r#"<artist id="{}" name="{}"/>"#,
                xml_escape(a["id"].as_str().unwrap_or("")),
                xml_escape(a["name"].as_str().unwrap_or(""))
            )
        })
        .collect();
    let albums_xml: String = albums.iter().map(|a| album_elem_xml(a)).collect();
    let songs_xml: String = songs.iter().map(|s| song_elem_xml(s)).collect();
    let xml = format!(r#"<searchResult3>{artists_xml}{albums_xml}{songs_xml}</searchResult3>"#);
    response::respond(f, json_data, &xml)
}

pub async fn scrobble(
    state: web::Data<SubsonicState>,
    query: web::Query<ScrobbleParams>,
) -> HttpResponse {
    let q = query.into_inner();
    let f = q.f.as_deref();
    if let Some(r) = auth_check(
        &state,
        q.u.as_deref(),
        q.p.as_deref(),
        q.t.as_deref(),
        q.s.as_deref(),
        f,
    ) {
        return r;
    }
    if let Some(id) = q.id.as_deref() {
        // Record play in track stats
        let _ = state.playlist_store.record_play(id).await;
    }
    response::respond(f, json!({}), "")
}

pub async fn get_now_playing(
    state: web::Data<SubsonicState>,
    query: web::Query<CommonParams>,
) -> HttpResponse {
    let q = query.into_inner();
    let f = q.f.as_deref();
    if let Some(r) = auth_check(
        &state,
        q.u.as_deref(),
        q.p.as_deref(),
        q.t.as_deref(),
        q.s.as_deref(),
        f,
    ) {
        return r;
    }
    let entries: Vec<Value> = match super::get_now_playing() {
        None => vec![],
        Some(info) => {
            let suffix = info.path.rsplit('.').next().unwrap_or("").to_lowercase();
            let content_type = mime_for_path(&info.path);
            let entry = json!({
                "id": info.track_id,
                "parent": info.album_id,
                "isDir": false,
                "title": info.title,
                "album": info.album,
                "artist": info.artist,
                "track": info.track_number,
                "year": if info.year > 0 { json!(info.year) } else { Value::Null },
                "coverArt": info.album_id.as_deref().map(|id| format!("al-{}", id)),
                "contentType": content_type,
                "suffix": suffix,
                "duration": info.length_ms / 1000,
                "bitRate": info.bitrate,
                "path": info.path,
                "isVideo": false,
                "albumId": info.album_id,
                "artistId": info.artist_id,
                "type": "music",
                "username": info.username,
                "minutesAgo": 0,
                "playerId": 1,
                "playerName": "Rockbox",
            });
            vec![entry]
        }
    };
    let json_data = json!({ "nowPlaying": { "entry": entries } });
    let xml_inner: String = entries
        .iter()
        .map(|e| {
            format!(
                r#"<entry id="{}" title="{}" artist="{}" album="{}" duration="{}" username="{}" minutesAgo="0" playerId="1"/>"#,
                xml_escape(e["id"].as_str().unwrap_or("")),
                xml_escape(e["title"].as_str().unwrap_or("")),
                xml_escape(e["artist"].as_str().unwrap_or("")),
                xml_escape(e["album"].as_str().unwrap_or("")),
                e["duration"].as_i64().unwrap_or(0),
                xml_escape(e["username"].as_str().unwrap_or(""))
            )
        })
        .collect();
    let xml = format!(r#"<nowPlaying>{xml_inner}</nowPlaying>"#);
    response::respond(f, json_data, &xml)
}

pub async fn update_now_playing(
    state: web::Data<SubsonicState>,
    query: web::Query<CommonParams>,
) -> HttpResponse {
    let q = query.into_inner();
    let f = q.f.as_deref();
    if let Some(r) = auth_check(
        &state,
        q.u.as_deref(),
        q.p.as_deref(),
        q.t.as_deref(),
        q.s.as_deref(),
        f,
    ) {
        return r;
    }
    response::respond(f, json!({}), "")
}

pub async fn get_album_list2(
    state: web::Data<SubsonicState>,
    query: web::Query<AlbumListParams>,
) -> HttpResponse {
    let q = query.into_inner();
    let f = q.f.as_deref();
    if let Some(r) = auth_check(
        &state,
        q.u.as_deref(),
        q.p.as_deref(),
        q.t.as_deref(),
        q.s.as_deref(),
        f,
    ) {
        return r;
    }
    let list_type = q.list_type.as_deref().unwrap_or("alphabeticalByName");
    let size = q.size.unwrap_or(10) as usize;
    let offset = q.offset.unwrap_or(0) as usize;

    let mut albums = repo::album::all(state.pool.clone())
        .await
        .unwrap_or_default();

    match list_type {
        "newest" => {
            // already ordered by created_at desc ideally; reverse as heuristic
            albums.reverse();
        }
        "alphabeticalByName" => {
            albums.sort_by(|a, b| a.title.cmp(&b.title));
        }
        "alphabeticalByArtist" => {
            albums.sort_by(|a, b| a.artist.cmp(&b.artist));
        }
        "byYear" => {
            let from = q.from_year.unwrap_or(0) as u32;
            let to = q.to_year.unwrap_or(u32::MAX as i64) as u32;
            let (lo, hi) = if from <= to { (from, to) } else { (to, from) };
            albums.retain(|a| a.year >= lo && a.year <= hi);
            if from <= to {
                albums.sort_by(|a, b| a.year.cmp(&b.year));
            } else {
                albums.sort_by(|a, b| b.year.cmp(&a.year));
            }
        }
        "random" => {
            let mut rng = rand::thread_rng();
            albums.shuffle(&mut rng);
        }
        "starred" => {
            // Return starred albums (favourites table with album_id set)
            let favs = repo::favourites::all_albums(state.pool.clone())
                .await
                .unwrap_or_default();
            let fav_ids: std::collections::HashSet<String> =
                favs.iter().map(|a| a.id.clone()).collect();
            albums.retain(|a| fav_ids.contains(&a.id));
        }
        _ => {}
    }

    let album_jsons: Vec<Value> = albums
        .iter()
        .skip(offset)
        .take(size)
        .map(|a| album_to_child(a, 0))
        .collect();

    let json_data = json!({ "albumList2": { "album": album_jsons } });
    let xml_inner: String = album_jsons.iter().map(|a| album_elem_xml(a)).collect();
    let xml = format!(r#"<albumList2>{xml_inner}</albumList2>"#);
    response::respond(f, json_data, &xml)
}

pub async fn get_random_songs(
    state: web::Data<SubsonicState>,
    query: web::Query<RandomSongsParams>,
) -> HttpResponse {
    let q = query.into_inner();
    let f = q.f.as_deref();
    if let Some(r) = auth_check(
        &state,
        q.u.as_deref(),
        q.p.as_deref(),
        q.t.as_deref(),
        q.s.as_deref(),
        f,
    ) {
        return r;
    }
    let size = q.size.unwrap_or(10) as usize;

    let mut tracks = repo::track::all(state.pool.clone())
        .await
        .unwrap_or_default();

    // Filter by year range if provided
    if let Some(from) = q.from_year {
        tracks.retain(|t| t.year.unwrap_or(0) as i64 >= from);
    }
    if let Some(to) = q.to_year {
        tracks.retain(|t| t.year.unwrap_or(9999) as i64 <= to);
    }
    // Filter by genre
    if let Some(ref genre) = q.genre {
        let genre_lc = genre.to_lowercase();
        tracks.retain(|t| {
            t.genre
                .as_deref()
                .map(|g| g.to_lowercase().contains(&genre_lc))
                .unwrap_or(false)
        });
    }

    let mut rng = rand::thread_rng();
    tracks.shuffle(&mut rng);
    tracks.truncate(size);

    let song_jsons: Vec<Value> = tracks.iter().map(track_to_child).collect();
    let json_data = json!({ "randomSongs": { "song": song_jsons } });
    let xml_inner: String = song_jsons.iter().map(|s| song_elem_xml(s)).collect();
    let xml = format!(r#"<randomSongs>{xml_inner}</randomSongs>"#);
    response::respond(f, json_data, &xml)
}

pub async fn get_playlists(
    state: web::Data<SubsonicState>,
    query: web::Query<CommonParams>,
) -> HttpResponse {
    let q = query.into_inner();
    let f = q.f.as_deref();
    if let Some(r) = auth_check(
        &state,
        q.u.as_deref(),
        q.p.as_deref(),
        q.t.as_deref(),
        q.s.as_deref(),
        f,
    ) {
        return r;
    }
    let playlists = state.playlist_store.list().await.unwrap_or_default();
    let pl_jsons: Vec<Value> = playlists
        .iter()
        .map(|p| {
            json!({
                "id": p.id,
                "name": p.name,
                "comment": p.description,
                "songCount": p.track_count,
                "duration": 0,
                "public": true,
                "coverArt": Value::Null,
            })
        })
        .collect();
    let json_data = json!({ "playlists": { "playlist": pl_jsons } });
    let xml_inner: String = pl_jsons
        .iter()
        .map(|p| {
            format!(
                r#"<playlist id="{}" name="{}" songCount="{}"/>"#,
                xml_escape(p["id"].as_str().unwrap_or("")),
                xml_escape(p["name"].as_str().unwrap_or("")),
                p["songCount"].as_i64().unwrap_or(0)
            )
        })
        .collect();
    let xml = format!(r#"<playlists>{xml_inner}</playlists>"#);
    response::respond(f, json_data, &xml)
}

pub async fn get_playlist(
    state: web::Data<SubsonicState>,
    query: web::Query<IdParam>,
) -> HttpResponse {
    let q = query.into_inner();
    let f = q.f.as_deref();
    if let Some(r) = auth_check(
        &state,
        q.u.as_deref(),
        q.p.as_deref(),
        q.t.as_deref(),
        q.s.as_deref(),
        f,
    ) {
        return r;
    }
    let id = match q.id.as_deref() {
        Some(id) => id,
        None => return response::respond_error(f, 10, "Required parameter is missing: id"),
    };
    let playlist = match state.playlist_store.get(id).await {
        Ok(Some(p)) => p,
        Ok(None) => return response::respond_error(f, 70, "Playlist not found"),
        Err(e) => {
            tracing::error!("getPlaylist: {e}");
            return response::respond_error(f, 0, "database error");
        }
    };
    let track_ids = state
        .playlist_store
        .get_track_ids(id)
        .await
        .unwrap_or_default();

    let mut entries: Vec<Value> = Vec::with_capacity(track_ids.len());
    for tid in &track_ids {
        if let Ok(Some(t)) = repo::track::find(state.pool.clone(), tid).await {
            entries.push(track_to_child(&t));
        }
    }

    let json_data = json!({
        "playlist": {
            "id": playlist.id,
            "name": playlist.name,
            "comment": playlist.description,
            "songCount": entries.len(),
            "duration": 0,
            "public": true,
            "entry": entries,
        }
    });
    let entries_xml: String = entries.iter().map(|s| song_elem_xml(s)).collect();
    let xml = format!(
        r#"<playlist id="{}" name="{}" songCount="{}">{entries_xml}</playlist>"#,
        xml_escape(&playlist.id),
        xml_escape(&playlist.name),
        entries.len()
    );
    response::respond(f, json_data, &xml)
}

pub async fn star(state: web::Data<SubsonicState>, query: web::Query<StarParams>) -> HttpResponse {
    let q = query.into_inner();
    let f = q.f.as_deref();
    if let Some(r) = auth_check(
        &state,
        q.u.as_deref(),
        q.p.as_deref(),
        q.t.as_deref(),
        q.s.as_deref(),
        f,
    ) {
        return r;
    }
    let track_id = q.id.clone();
    let album_id = q.album_id.clone();

    // Persist to local favourites
    let fav = Favourites {
        id: Uuid::new_v4().to_string(),
        track_id: track_id.clone(),
        album_id: album_id.clone(),
        created_at: chrono::Utc::now(),
    };
    if let Err(e) = repo::favourites::save(state.pool.clone(), fav).await {
        tracing::error!("star save: {e}");
    }

    // Mirror to Rocksky if a token exists and we have a track id
    if let Some(ref tid) = track_id {
        let pool = state.pool.clone();
        let tid = tid.clone();
        tokio::spawn(async move {
            let track = match repo::track::find(pool.clone(), &tid).await {
                Ok(Some(t)) => t,
                _ => return,
            };
            let album = match repo::album::find(pool, &track.album_id).await {
                Ok(Some(a)) => a,
                _ => return,
            };
            if let Err(e) = rockbox_rocksky::like(track, album).await {
                tracing::warn!("star rocksky like: {e}");
            }
        });
    }

    response::respond(f, json!({}), "")
}

pub async fn unstar(
    state: web::Data<SubsonicState>,
    query: web::Query<StarParams>,
) -> HttpResponse {
    let q = query.into_inner();
    let f = q.f.as_deref();
    if let Some(r) = auth_check(
        &state,
        q.u.as_deref(),
        q.p.as_deref(),
        q.t.as_deref(),
        q.s.as_deref(),
        f,
    ) {
        return r;
    }

    // id may refer to a track, album_id, or artist_id
    let track_id = q.id.clone();
    let fav_id = track_id
        .as_deref()
        .or(q.album_id.as_deref())
        .or(q.artist_id.as_deref())
        .unwrap_or("");

    if !fav_id.is_empty() {
        if let Err(e) = repo::favourites::delete(state.pool.clone(), fav_id).await {
            tracing::error!("unstar delete: {e}");
        }
    }

    // Mirror removal to Rocksky if we have a track id
    if let Some(ref tid) = track_id {
        let pool = state.pool.clone();
        let tid = tid.clone();
        tokio::spawn(async move {
            let track = match repo::track::find(pool, &tid).await {
                Ok(Some(t)) => t,
                _ => return,
            };
            if let Err(e) = rockbox_rocksky::unlike(track).await {
                tracing::warn!("unstar rocksky unlike: {e}");
            }
        });
    }

    response::respond(f, json!({}), "")
}

// ── Artist / Album info ───────────────────────────────────────────────────────

pub async fn get_artist_info2(
    state: web::Data<SubsonicState>,
    query: web::Query<ArtistInfoParams>,
) -> HttpResponse {
    let q = query.into_inner();
    let f = q.f.as_deref();
    if let Some(r) = auth_check(
        &state,
        q.u.as_deref(),
        q.p.as_deref(),
        q.t.as_deref(),
        q.s.as_deref(),
        f,
    ) {
        return r;
    }
    let id = match q.id.as_deref() {
        Some(id) => id,
        None => return response::respond_error(f, 10, "Required parameter is missing: id"),
    };
    let (bio, image_url) = match repo::artist::find(state.pool.clone(), id).await {
        Ok(Some(a)) => (a.bio.unwrap_or_default(), a.image.unwrap_or_default()),
        _ => (String::new(), String::new()),
    };
    let json_data = json!({
        "artistInfo2": {
            "biography": bio,
            "musicBrainzId": "",
            "lastFmUrl": "",
            "smallImageUrl": image_url,
            "mediumImageUrl": image_url,
            "largeImageUrl": image_url,
            "similarArtist": [],
        }
    });
    let xml = format!(
        r#"<artistInfo2><biography>{}</biography><musicBrainzId/><lastFmUrl/><smallImageUrl>{}</smallImageUrl><mediumImageUrl>{}</mediumImageUrl><largeImageUrl>{}</largeImageUrl></artistInfo2>"#,
        xml_escape(&bio),
        xml_escape(&image_url),
        xml_escape(&image_url),
        xml_escape(&image_url)
    );
    response::respond(f, json_data, &xml)
}

pub async fn get_artist_info(
    state: web::Data<SubsonicState>,
    query: web::Query<ArtistInfoParams>,
) -> HttpResponse {
    let q = query.into_inner();
    let f = q.f.as_deref();
    if let Some(r) = auth_check(
        &state,
        q.u.as_deref(),
        q.p.as_deref(),
        q.t.as_deref(),
        q.s.as_deref(),
        f,
    ) {
        return r;
    }
    let id = match q.id.as_deref() {
        Some(id) => id,
        None => return response::respond_error(f, 10, "Required parameter is missing: id"),
    };
    let (bio, image_url) = match repo::artist::find(state.pool.clone(), id).await {
        Ok(Some(a)) => (a.bio.unwrap_or_default(), a.image.unwrap_or_default()),
        _ => (String::new(), String::new()),
    };
    let json_data = json!({
        "artistInfo": {
            "biography": bio,
            "musicBrainzId": "",
            "lastFmUrl": "",
            "smallImageUrl": image_url,
            "mediumImageUrl": image_url,
            "largeImageUrl": image_url,
            "similarArtist": [],
        }
    });
    response::respond(f, json_data, "")
}

pub async fn get_album_info2(
    state: web::Data<SubsonicState>,
    query: web::Query<IdParam>,
) -> HttpResponse {
    let q = query.into_inner();
    let f = q.f.as_deref();
    if let Some(r) = auth_check(
        &state,
        q.u.as_deref(),
        q.p.as_deref(),
        q.t.as_deref(),
        q.s.as_deref(),
        f,
    ) {
        return r;
    }
    let json_data = json!({
        "albumInfo": {
            "notes": "",
            "musicBrainzId": "",
            "lastFmUrl": "",
            "smallImageUrl": "",
            "mediumImageUrl": "",
            "largeImageUrl": "",
        }
    });
    response::respond(f, json_data, "<albumInfo/>")
}

pub async fn get_similar_songs2(
    state: web::Data<SubsonicState>,
    query: web::Query<IdParam>,
) -> HttpResponse {
    let q = query.into_inner();
    let f = q.f.as_deref();
    if let Some(r) = auth_check(
        &state,
        q.u.as_deref(),
        q.p.as_deref(),
        q.t.as_deref(),
        q.s.as_deref(),
        f,
    ) {
        return r;
    }
    response::respond(
        f,
        json!({ "similarSongs2": { "song": [] } }),
        "<similarSongs2/>",
    )
}

pub async fn get_top_songs(
    state: web::Data<SubsonicState>,
    query: web::Query<TopSongsParams>,
) -> HttpResponse {
    let q = query.into_inner();
    let f = q.f.as_deref();
    if let Some(r) = auth_check(
        &state,
        q.u.as_deref(),
        q.p.as_deref(),
        q.t.as_deref(),
        q.s.as_deref(),
        f,
    ) {
        return r;
    }
    let count = q.count.unwrap_or(50) as usize;
    let artist_name = q.artist.as_deref().unwrap_or("");
    let mut tracks = if artist_name.is_empty() {
        vec![]
    } else {
        let all = repo::track::all(state.pool.clone())
            .await
            .unwrap_or_default();
        all.into_iter()
            .filter(|t| t.artist.eq_ignore_ascii_case(artist_name))
            .take(count)
            .collect::<Vec<_>>()
    };
    tracks.truncate(count);
    let songs: Vec<Value> = tracks.iter().map(track_to_child).collect();
    let xml_inner: String = songs.iter().map(|s| song_elem_xml(s)).collect();
    response::respond(
        f,
        json!({ "topSongs": { "song": songs } }),
        &format!("<topSongs>{xml_inner}</topSongs>"),
    )
}

pub async fn get_lyrics(
    state: web::Data<SubsonicState>,
    query: web::Query<LyricsParams>,
) -> HttpResponse {
    let q = query.into_inner();
    let f = q.f.as_deref();
    if let Some(r) = auth_check(
        &state,
        q.u.as_deref(),
        q.p.as_deref(),
        q.t.as_deref(),
        q.s.as_deref(),
        f,
    ) {
        return r;
    }
    let json_data = json!({ "lyrics": { "artist": "", "title": "", "value": "" } });
    response::respond(f, json_data, r#"<lyrics artist="" title=""/>"#)
}

// ── Aliases for older/folder-browsing API calls ───────────────────────────────

pub async fn get_album_list(
    state: web::Data<SubsonicState>,
    query: web::Query<AlbumListParams>,
) -> HttpResponse {
    get_album_list2(state, query).await
}

pub async fn get_starred(
    state: web::Data<SubsonicState>,
    query: web::Query<CommonParams>,
) -> HttpResponse {
    get_starred2(state, query).await
}

pub async fn search2(
    state: web::Data<SubsonicState>,
    query: web::Query<SearchParams>,
) -> HttpResponse {
    let q = query.into_inner();
    let f = q.f.as_deref();
    if let Some(r) = auth_check(
        &state,
        q.u.as_deref(),
        q.p.as_deref(),
        q.t.as_deref(),
        q.s.as_deref(),
        f,
    ) {
        return r;
    }
    let term = q.query.as_deref().unwrap_or("").to_lowercase();
    let artist_limit = q.artist_count.unwrap_or(20) as usize;
    let album_limit = q.album_count.unwrap_or(20) as usize;
    let song_limit = q.song_count.unwrap_or(20) as usize;
    let artist_offset = q.artist_offset.unwrap_or(0) as usize;
    let album_offset = q.album_offset.unwrap_or(0) as usize;
    let song_offset = q.song_offset.unwrap_or(0) as usize;

    let all_artists = repo::artist::all(state.pool.clone())
        .await
        .unwrap_or_default();
    let artists: Vec<Value> = all_artists
        .iter()
        .filter(|a| term.is_empty() || a.name.to_lowercase().contains(&term))
        .skip(artist_offset)
        .take(artist_limit)
        .map(|a| artist_to_json(a, 0))
        .collect();

    let all_albums = repo::album::all(state.pool.clone())
        .await
        .unwrap_or_default();
    let albums: Vec<Value> = all_albums
        .iter()
        .filter(|a| {
            term.is_empty()
                || a.title.to_lowercase().contains(&term)
                || a.artist.to_lowercase().contains(&term)
        })
        .skip(album_offset)
        .take(album_limit)
        .map(|a| album_to_child(a, 0))
        .collect();

    let all_tracks = repo::track::all(state.pool.clone())
        .await
        .unwrap_or_default();
    let songs: Vec<Value> = all_tracks
        .iter()
        .filter(|t| {
            term.is_empty()
                || t.title.to_lowercase().contains(&term)
                || t.artist.to_lowercase().contains(&term)
                || t.album.to_lowercase().contains(&term)
        })
        .skip(song_offset)
        .take(song_limit)
        .map(track_to_child)
        .collect();

    let json_data = json!({
        "searchResult2": {
            "artist": artists,
            "album": albums,
            "song": songs,
        }
    });
    response::respond(f, json_data, "")
}

// ── XML helpers ───────────────────────────────────────────────────────────────

fn xml_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

fn safe_filename(path: &str) -> String {
    path.rsplit('/').next().unwrap_or("audio").to_string()
}

fn album_elem_xml(a: &Value) -> String {
    format!(
        r#"<album id="{}" name="{}" artist="{}" artistId="{}" songCount="{}" coverArt="{}"/>"#,
        xml_escape(a["id"].as_str().unwrap_or("")),
        xml_escape(a["name"].as_str().unwrap_or("")),
        xml_escape(a["artist"].as_str().unwrap_or("")),
        xml_escape(a["artistId"].as_str().unwrap_or("")),
        a["songCount"].as_i64().unwrap_or(0),
        xml_escape(a["coverArt"].as_str().unwrap_or(""))
    )
}

fn song_elem_xml(s: &Value) -> String {
    format!(
        r#"<song id="{}" parent="{}" title="{}" album="{}" artist="{}" isDir="false" coverArt="{}" duration="{}" bitRate="{}" track="{}" contentType="{}" suffix="{}" albumId="{}" artistId="{}"/>"#,
        xml_escape(s["id"].as_str().unwrap_or("")),
        xml_escape(s["parent"].as_str().unwrap_or("")),
        xml_escape(s["title"].as_str().unwrap_or("")),
        xml_escape(s["album"].as_str().unwrap_or("")),
        xml_escape(s["artist"].as_str().unwrap_or("")),
        xml_escape(s["coverArt"].as_str().unwrap_or("")),
        s["duration"].as_i64().unwrap_or(0),
        s["bitRate"].as_i64().unwrap_or(0),
        s["track"].as_i64().unwrap_or(0),
        xml_escape(s["contentType"].as_str().unwrap_or("audio/mpeg")),
        xml_escape(s["suffix"].as_str().unwrap_or("")),
        xml_escape(s["albumId"].as_str().unwrap_or("")),
        xml_escape(s["artistId"].as_str().unwrap_or(""))
    )
}

// ── Multi-value query param helper ───────────────────────────────────────────

/// Extract all values for `key` from a raw query string (e.g. `id=1&id=2&id=3`).
/// IDs are UUIDs/integers so no percent-decoding is needed.
fn multi_param(query: &str, key: &str) -> Vec<String> {
    let prefix = format!("{}=", key);
    query
        .split('&')
        .filter_map(|part| part.strip_prefix(&prefix).map(|v| v.to_string()))
        .collect()
}

/// Extract the first value for `key` from a raw query string.
fn single_param<'a>(query: &'a str, key: &str) -> Option<&'a str> {
    let prefix = format!("{}=", key);
    query
        .split('&')
        .find_map(|part| part.strip_prefix(prefix.as_str()))
}

// ── New handlers ──────────────────────────────────────────────────────────────

pub async fn get_user(
    state: web::Data<SubsonicState>,
    query: web::Query<CommonParams>,
) -> HttpResponse {
    let q = query.into_inner();
    let f = q.f.as_deref();
    if let Some(r) = auth_check(
        &state,
        q.u.as_deref(),
        q.p.as_deref(),
        q.t.as_deref(),
        q.s.as_deref(),
        f,
    ) {
        return r;
    }
    let username = state.username.as_str();
    let json_data = json!({
        "user": {
            "username": username,
            "email": "",
            "scrobblingEnabled": true,
            "maxBitRate": 0,
            "adminRole": true,
            "settingsRole": true,
            "downloadRole": true,
            "uploadRole": false,
            "playlistRole": true,
            "coverArtRole": true,
            "commentRole": false,
            "podcastRole": false,
            "streamRole": true,
            "jukeboxRole": false,
            "shareRole": false,
            "videoConversionRole": false,
            "folder": [1],
        }
    });
    let xml = format!(
        r#"<user username="{}" scrobblingEnabled="true" adminRole="true" settingsRole="true" downloadRole="true" uploadRole="false" playlistRole="true" coverArtRole="true" commentRole="false" podcastRole="false" streamRole="true" jukeboxRole="false" shareRole="false"><folder>1</folder></user>"#,
        xml_escape(username)
    );
    response::respond(f, json_data, &xml)
}

pub async fn get_scan_status(
    state: web::Data<SubsonicState>,
    query: web::Query<CommonParams>,
) -> HttpResponse {
    let q = query.into_inner();
    let f = q.f.as_deref();
    if let Some(r) = auth_check(
        &state,
        q.u.as_deref(),
        q.p.as_deref(),
        q.t.as_deref(),
        q.s.as_deref(),
        f,
    ) {
        return r;
    }
    let scanning = state.scan_running.load(Ordering::Relaxed);
    let json_data = json!({ "scanStatus": { "scanning": scanning, "count": 0 } });
    let xml = format!(r#"<scanStatus scanning="{}" count="0"/>"#, scanning);
    response::respond(f, json_data, &xml)
}

pub async fn start_scan(
    state: web::Data<SubsonicState>,
    query: web::Query<CommonParams>,
) -> HttpResponse {
    let q = query.into_inner();
    let f = q.f.as_deref();
    if let Some(r) = auth_check(
        &state,
        q.u.as_deref(),
        q.p.as_deref(),
        q.t.as_deref(),
        q.s.as_deref(),
        f,
    ) {
        return r;
    }
    // Only start one scan at a time
    if !state
        .scan_running
        .compare_exchange(false, true, Ordering::SeqCst, Ordering::Relaxed)
        .is_ok()
    {
        let json_data = json!({ "scanStatus": { "scanning": true, "count": 0 } });
        return response::respond(f, json_data, r#"<scanStatus scanning="true" count="0"/>"#);
    }
    let pool = state.pool.clone();
    let flag = state.scan_running.clone();
    tokio::spawn(async move {
        let home = std::env::var("HOME").unwrap_or_default();
        let path = std::env::var("ROCKBOX_LIBRARY").unwrap_or_else(|_| format!("{}/Music", home));
        if let Err(e) = scan_audio_files(pool, path.into()).await {
            tracing::error!("startScan: {e}");
        }
        flag.store(false, Ordering::Relaxed);
    });
    let json_data = json!({ "scanStatus": { "scanning": true, "count": 0 } });
    response::respond(f, json_data, r#"<scanStatus scanning="true" count="0"/>"#)
}

pub async fn get_indexes(
    state: web::Data<SubsonicState>,
    query: web::Query<CommonParams>,
) -> HttpResponse {
    let q = query.into_inner();
    let f = q.f.as_deref();
    if let Some(r) = auth_check(
        &state,
        q.u.as_deref(),
        q.p.as_deref(),
        q.t.as_deref(),
        q.s.as_deref(),
        f,
    ) {
        return r;
    }
    let artists = repo::artist::all(state.pool.clone())
        .await
        .unwrap_or_default();
    let album_counts = repo::album::count_by_artist(state.pool.clone())
        .await
        .unwrap_or_default();
    let index = build_artist_index(&artists, &album_counts);
    let json_data = json!({
        "indexes": {
            "lastModified": 0,
            "ignoredArticles": "The An A Die Das Ein",
            "index": index,
        }
    });
    let xml_entries: String = index
        .iter()
        .map(|g| {
            let name = g["name"].as_str().unwrap_or("");
            let inner: String = g["artist"]
                .as_array()
                .map(|arr| {
                    arr.iter()
                        .map(|a| {
                            format!(
                                r#"<artist id="{}" name="{}"/>"#,
                                xml_escape(a["id"].as_str().unwrap_or("")),
                                xml_escape(a["name"].as_str().unwrap_or(""))
                            )
                        })
                        .collect()
                })
                .unwrap_or_default();
            format!(r#"<index name="{}">{}</index>"#, xml_escape(name), inner)
        })
        .collect();
    let xml = format!(
        r#"<indexes lastModified="0" ignoredArticles="The An A Die Das Ein">{xml_entries}</indexes>"#
    );
    response::respond(f, json_data, &xml)
}

pub async fn get_music_directory(
    state: web::Data<SubsonicState>,
    query: web::Query<IdParam>,
) -> HttpResponse {
    let q = query.into_inner();
    let f = q.f.as_deref();
    if let Some(r) = auth_check(
        &state,
        q.u.as_deref(),
        q.p.as_deref(),
        q.t.as_deref(),
        q.s.as_deref(),
        f,
    ) {
        return r;
    }
    let id = match q.id.as_deref() {
        Some(id) => id,
        None => return response::respond_error(f, 10, "Required parameter is missing: id"),
    };

    // id == "1" means the root music folder → list all artists
    if id == "1" {
        let artists = repo::artist::all(state.pool.clone())
            .await
            .unwrap_or_default();
        let children: Vec<Value> = artists
            .iter()
            .map(|a| {
                json!({
                    "id": a.id,
                    "parent": "1",
                    "isDir": true,
                    "title": a.name,
                    "album": a.name,
                    "artist": a.name,
                    "coverArt": format!("ar-{}", a.id),
                })
            })
            .collect();
        let xml_children: String = children
            .iter()
            .map(|c| {
                format!(
                    r#"<child id="{}" parent="1" isDir="true" title="{}" coverArt="{}"/>"#,
                    xml_escape(c["id"].as_str().unwrap_or("")),
                    xml_escape(c["title"].as_str().unwrap_or("")),
                    xml_escape(c["coverArt"].as_str().unwrap_or(""))
                )
            })
            .collect();
        let json_data = json!({ "directory": { "id": "1", "name": "Music", "child": children } });
        let xml = format!(r#"<directory id="1" name="Music">{xml_children}</directory>"#);
        return response::respond(f, json_data, &xml);
    }

    // Try as artist id → return albums
    if let Ok(Some(artist)) = repo::artist::find(state.pool.clone(), id).await {
        let albums = repo::album::find_by_artist(state.pool.clone(), id)
            .await
            .unwrap_or_default();
        let children: Vec<Value> = albums
            .iter()
            .map(|a| {
                json!({
                    "id": a.id,
                    "parent": id,
                    "isDir": true,
                    "title": a.title,
                    "album": a.title,
                    "artist": a.artist,
                    "year": if a.year > 0 { json!(a.year) } else { Value::Null },
                    "coverArt": format!("al-{}", a.id),
                })
            })
            .collect();
        let xml_children: String = children.iter().map(|c| format!(
            r#"<child id="{}" parent="{}" isDir="true" title="{}" artist="{}" coverArt="{}"/>"#,
            xml_escape(c["id"].as_str().unwrap_or("")),
            xml_escape(id),
            xml_escape(c["title"].as_str().unwrap_or("")),
            xml_escape(c["artist"].as_str().unwrap_or("")),
            xml_escape(c["coverArt"].as_str().unwrap_or(""))
        )).collect();
        let json_data = json!({
            "directory": { "id": id, "name": artist.name, "child": children }
        });
        let xml = format!(
            r#"<directory id="{}" name="{}">{xml_children}</directory>"#,
            xml_escape(id),
            xml_escape(&artist.name)
        );
        return response::respond(f, json_data, &xml);
    }

    // Try as album id → return tracks
    if let Ok(Some(album)) = repo::album::find(state.pool.clone(), id).await {
        let tracks = repo::album_tracks::find_by_album(state.pool.clone(), id)
            .await
            .unwrap_or_default();
        let children: Vec<Value> = tracks.iter().map(track_to_child).collect();
        let xml_children: String = children.iter().map(|s| song_elem_xml(s)).collect();
        let json_data = json!({
            "directory": { "id": id, "name": album.title, "child": children }
        });
        let xml = format!(
            r#"<directory id="{}" name="{}">{xml_children}</directory>"#,
            xml_escape(id),
            xml_escape(&album.title)
        );
        return response::respond(f, json_data, &xml);
    }

    response::respond_error(f, 70, "Directory not found")
}

pub async fn get_genres(
    state: web::Data<SubsonicState>,
    query: web::Query<CommonParams>,
) -> HttpResponse {
    let q = query.into_inner();
    let f = q.f.as_deref();
    if let Some(r) = auth_check(
        &state,
        q.u.as_deref(),
        q.p.as_deref(),
        q.t.as_deref(),
        q.s.as_deref(),
        f,
    ) {
        return r;
    }
    let genres = repo::genre::all(state.pool.clone())
        .await
        .unwrap_or_default();
    let genre_jsons: Vec<Value> = genres
        .iter()
        .map(|g| {
            json!({
                "value": g.name,
                "songCount": 0,
                "albumCount": 0,
            })
        })
        .collect();
    let json_data = json!({ "genres": { "genre": genre_jsons } });
    let xml_inner: String = genres
        .iter()
        .map(|g| {
            format!(
                r#"<genre songCount="0" albumCount="0">{}</genre>"#,
                xml_escape(&g.name)
            )
        })
        .collect();
    let xml = format!(r#"<genres>{xml_inner}</genres>"#);
    response::respond(f, json_data, &xml)
}

pub async fn get_songs_by_genre(
    state: web::Data<SubsonicState>,
    query: web::Query<SongsByGenreParams>,
) -> HttpResponse {
    let q = query.into_inner();
    let f = q.f.as_deref();
    if let Some(r) = auth_check(
        &state,
        q.u.as_deref(),
        q.p.as_deref(),
        q.t.as_deref(),
        q.s.as_deref(),
        f,
    ) {
        return r;
    }
    let genre_name = match q.genre.as_deref() {
        Some(g) => g,
        None => return response::respond_error(f, 10, "Required parameter is missing: genre"),
    };
    let count = q.count.unwrap_or(10) as usize;
    let offset = q.offset.unwrap_or(0) as usize;

    // Look up genre by name (case-insensitive)
    let genres = repo::genre::all(state.pool.clone())
        .await
        .unwrap_or_default();
    let genre = genres
        .iter()
        .find(|g| g.name.eq_ignore_ascii_case(genre_name));
    let songs: Vec<Value> = match genre {
        Some(g) => {
            let tracks = repo::genre::find_tracks(state.pool.clone(), &g.id)
                .await
                .unwrap_or_default();
            tracks
                .iter()
                .skip(offset)
                .take(count)
                .map(track_to_child)
                .collect()
        }
        None => vec![],
    };
    let json_data = json!({ "songsByGenre": { "song": songs } });
    let xml_inner: String = songs.iter().map(|s| song_elem_xml(s)).collect();
    let xml = format!(r#"<songsByGenre>{xml_inner}</songsByGenre>"#);
    response::respond(f, json_data, &xml)
}

pub async fn get_starred2(
    state: web::Data<SubsonicState>,
    query: web::Query<CommonParams>,
) -> HttpResponse {
    let q = query.into_inner();
    let f = q.f.as_deref();
    if let Some(r) = auth_check(
        &state,
        q.u.as_deref(),
        q.p.as_deref(),
        q.t.as_deref(),
        q.s.as_deref(),
        f,
    ) {
        return r;
    }

    let fav_tracks = repo::favourites::all_tracks(state.pool.clone())
        .await
        .unwrap_or_default();
    let fav_albums = repo::favourites::all_albums(state.pool.clone())
        .await
        .unwrap_or_default();

    let starred_ts = chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string();
    let songs: Vec<Value> = fav_tracks
        .iter()
        .map(|t| {
            let mut s = track_to_child(t);
            if let Some(obj) = s.as_object_mut() {
                obj.insert("starred".to_string(), json!(starred_ts));
            }
            s
        })
        .collect();
    let albums: Vec<Value> = fav_albums
        .iter()
        .map(|a| {
            let mut obj = album_to_child(a, 0);
            if let Some(m) = obj.as_object_mut() {
                m.insert("starred".to_string(), json!(starred_ts));
            }
            obj
        })
        .collect();

    let json_data = json!({
        "starred2": {
            "artist": [],
            "album": albums,
            "song": songs,
        }
    });
    let songs_xml: String = songs.iter().map(|s| song_elem_xml(s)).collect();
    let albums_xml: String = albums.iter().map(|a| album_elem_xml(a)).collect();
    let xml = format!(r#"<starred2>{albums_xml}{songs_xml}</starred2>"#);
    response::respond(f, json_data, &xml)
}

pub async fn create_playlist(state: web::Data<SubsonicState>, req: HttpRequest) -> HttpResponse {
    let qs = req.query_string();
    let u = single_param(qs, "u");
    let p = single_param(qs, "p");
    let t = single_param(qs, "t");
    let s = single_param(qs, "s");
    let f = single_param(qs, "f");
    if !super::check_auth(&state.username, &state.password, u, p, t, s) {
        return response::respond_error(f, 40, "Wrong username or password");
    }

    let playlist_id = single_param(qs, "playlistId");
    let name = single_param(qs, "name").unwrap_or("Untitled");
    let song_ids = multi_param(qs, "songId");

    if let Some(pid) = playlist_id {
        // Update existing playlist's tracks
        if let Err(e) = state.playlist_store.add_tracks(pid, &song_ids).await {
            tracing::error!("createPlaylist update: {e}");
            return response::respond_error(f, 0, "database error");
        }
        return get_playlist_by_id(&state, pid, f).await;
    }

    // Create new playlist
    match state.playlist_store.create(name, None, None, None).await {
        Ok(pl) => {
            if !song_ids.is_empty() {
                let _ = state.playlist_store.add_tracks(&pl.id, &song_ids).await;
            }
            get_playlist_by_id(&state, &pl.id, f).await
        }

        Err(e) => {
            tracing::error!("createPlaylist: {e}");
            response::respond_error(f, 0, "database error")
        }
    }
}

pub async fn update_playlist(state: web::Data<SubsonicState>, req: HttpRequest) -> HttpResponse {
    let qs = req.query_string();
    let u = single_param(qs, "u");
    let p = single_param(qs, "p");
    let t = single_param(qs, "t");
    let s = single_param(qs, "s");
    let f = single_param(qs, "f");
    if !super::check_auth(&state.username, &state.password, u, p, t, s) {
        return response::respond_error(f, 40, "Wrong username or password");
    }

    let pid = match single_param(qs, "playlistId") {
        Some(id) => id,
        None => return response::respond_error(f, 10, "Required parameter is missing: playlistId"),
    };

    // Rename if requested
    if let Some(new_name) = single_param(qs, "name") {
        let comment = single_param(qs, "comment").map(|s| s.to_string());
        if let Err(e) = state
            .playlist_store
            .update(pid, new_name, comment.as_deref(), None, None)
            .await
        {
            tracing::error!("updatePlaylist rename: {e}");
        }
    }

    // Add songs
    let to_add = multi_param(qs, "songIdToAdd");
    if !to_add.is_empty() {
        if let Err(e) = state.playlist_store.add_tracks(pid, &to_add).await {
            tracing::error!("updatePlaylist add: {e}");
        }
    }

    // Remove songs by position index
    let indices = multi_param(qs, "songIndexToRemove");
    if !indices.is_empty() {
        let track_ids = state
            .playlist_store
            .get_track_ids(pid)
            .await
            .unwrap_or_default();
        for idx_str in &indices {
            if let Ok(idx) = idx_str.parse::<usize>() {
                if let Some(tid) = track_ids.get(idx) {
                    let _ = state.playlist_store.remove_track(pid, tid).await;
                }
            }
        }
    }

    response::respond(f, json!({}), "")
}

pub async fn delete_playlist(
    state: web::Data<SubsonicState>,
    query: web::Query<IdParam>,
) -> HttpResponse {
    let q = query.into_inner();
    let f = q.f.as_deref();
    if let Some(r) = auth_check(
        &state,
        q.u.as_deref(),
        q.p.as_deref(),
        q.t.as_deref(),
        q.s.as_deref(),
        f,
    ) {
        return r;
    }
    let id = match q.id.as_deref() {
        Some(id) => id,
        None => return response::respond_error(f, 10, "Required parameter is missing: id"),
    };
    if let Err(e) = state.playlist_store.delete(id).await {
        tracing::error!("deletePlaylist: {e}");
        return response::respond_error(f, 0, "database error");
    }
    response::respond(f, json!({}), "")
}

// ── Internal helpers ──────────────────────────────────────────────────────────

async fn get_playlist_by_id(state: &SubsonicState, id: &str, f: Option<&str>) -> HttpResponse {
    let playlist = match state.playlist_store.get(id).await {
        Ok(Some(p)) => p,
        Ok(None) => return response::respond_error(f, 70, "Playlist not found"),
        Err(e) => {
            tracing::error!("get_playlist_by_id: {e}");
            return response::respond_error(f, 0, "database error");
        }
    };
    let track_ids = state
        .playlist_store
        .get_track_ids(id)
        .await
        .unwrap_or_default();
    let mut entries: Vec<Value> = Vec::with_capacity(track_ids.len());
    for tid in &track_ids {
        if let Ok(Some(t)) = repo::track::find(state.pool.clone(), tid).await {
            entries.push(track_to_child(&t));
        }
    }
    let json_data = json!({
        "playlist": {
            "id": playlist.id,
            "name": playlist.name,
            "comment": playlist.description,
            "songCount": entries.len(),
            "duration": 0,
            "public": true,
            "entry": entries,
        }
    });
    let entries_xml: String = entries.iter().map(|s| song_elem_xml(s)).collect();
    let xml = format!(
        r#"<playlist id="{}" name="{}" songCount="{}">{entries_xml}</playlist>"#,
        xml_escape(&playlist.id),
        xml_escape(&playlist.name),
        entries.len()
    );
    response::respond(f, json_data, &xml)
}
