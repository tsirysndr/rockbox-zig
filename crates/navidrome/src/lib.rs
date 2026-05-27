#[cfg(feature = "server")]
pub mod server;

use serde::Deserialize;
use std::time::Duration;

const API_VERSION: &str = "1.16.1";
const CLIENT_NAME: &str = "rockbox";

#[derive(Debug, Clone)]
pub struct NavidromeEntry {
    pub id: String,
    pub name: String,
    pub is_container: bool,
    pub stream_url: Option<String>,
}

// ── Public rich types ─────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct NavidromeAlbum {
    pub id: String,
    pub name: String,
    pub artist: String,
    pub artist_id: String,
    pub year: Option<u32>,
    pub cover_art: Option<String>,
    pub song_count: u32,
    pub duration: u32,
}

#[derive(Debug, Clone)]
pub struct NavidromeArtist {
    pub id: String,
    pub name: String,
    pub cover_art: Option<String>,
    pub album_count: u32,
}

#[derive(Debug, Clone)]
pub struct NavidromeGenre {
    pub name: String,
    pub song_count: u32,
    pub album_count: u32,
}

#[derive(Debug, Clone)]
pub struct NavidromePlaylist {
    pub id: String,
    pub name: String,
    pub comment: Option<String>,
    pub song_count: u32,
    pub cover_art: Option<String>,
}

#[derive(Debug, Clone)]
pub struct NavidromeSong {
    pub id: String,
    pub title: String,
    pub artist: String,
    pub artist_id: String,
    pub album: String,
    pub album_id: String,
    pub cover_art: Option<String>,
    pub duration: u32,
    pub track: Option<u32>,
    pub year: Option<u32>,
}

// ── Subsonic JSON response wrappers ──────────────────────────────────────────

#[derive(Debug, Deserialize)]
struct SubsonicEnvelope {
    #[serde(rename = "subsonic-response")]
    response: SubsonicBody,
}

#[derive(Debug, Deserialize)]
struct SubsonicBody {
    status: String,
    #[serde(rename = "musicFolders")]
    music_folders: Option<MusicFoldersWrapper>,
    indexes: Option<IndexesWrapper>,
    directory: Option<Directory>,
    #[serde(rename = "albumList2")]
    album_list2: Option<AlbumList2Wrapper>,
    album: Option<AlbumDetailBody>,
    artists: Option<ArtistsWrapper>,
    artist: Option<ArtistDetailBody>,
    genres: Option<GenresWrapper>,
    #[serde(rename = "songsByGenre")]
    songs_by_genre: Option<SongsByGenreWrapper>,
    playlists: Option<PlaylistsWrapper>,
    playlist: Option<PlaylistDetailBody>,
    starred2: Option<Starred2Wrapper>,
    #[serde(rename = "searchResult3")]
    search_result3: Option<SearchResult3>,
    song: Option<SongEntry>,
}

#[derive(Debug, Deserialize)]
struct MusicFoldersWrapper {
    #[serde(rename = "musicFolder")]
    music_folder: Vec<MusicFolder>,
}

#[derive(Debug, Deserialize)]
struct MusicFolder {
    id: serde_json::Value, // int or string depending on server
    name: Option<String>,
}

impl MusicFolder {
    fn id_str(&self) -> String {
        match &self.id {
            serde_json::Value::Number(n) => n.to_string(),
            serde_json::Value::String(s) => s.clone(),
            v => v.to_string(),
        }
    }
}

#[derive(Debug, Deserialize)]
struct IndexesWrapper {
    index: Option<Vec<IndexLetter>>,
}

#[derive(Debug, Deserialize)]
struct IndexLetter {
    artist: Option<Vec<ArtistEntry>>,
}

#[derive(Debug, Deserialize)]
struct ArtistEntry {
    id: String,
    name: String,
}

#[derive(Debug, Deserialize)]
struct Directory {
    child: Option<Vec<DirectoryChild>>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct DirectoryChild {
    id: String,
    title: Option<String>,
    name: Option<String>,
    is_dir: Option<bool>,
}

impl DirectoryChild {
    fn display_name(&self) -> String {
        self.title
            .clone()
            .or_else(|| self.name.clone())
            .unwrap_or_else(|| self.id.clone())
    }
}

// ── New endpoint serde types ──────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
struct AlbumList2Wrapper {
    album: Option<Vec<AlbumEntry>>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct AlbumEntry {
    id: String,
    name: String,
    #[serde(default)]
    artist: String,
    #[serde(default, rename = "artistId")]
    artist_id: String,
    year: Option<u32>,
    #[serde(rename = "coverArt")]
    cover_art: Option<String>,
    #[serde(default, rename = "songCount")]
    song_count: u32,
    #[serde(default)]
    duration: u32,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct AlbumDetailBody {
    id: String,
    name: String,
    #[serde(default)]
    artist: String,
    #[serde(default, rename = "artistId")]
    artist_id: String,
    year: Option<u32>,
    #[serde(rename = "coverArt")]
    cover_art: Option<String>,
    #[serde(default, rename = "songCount")]
    song_count: u32,
    #[serde(default)]
    duration: u32,
    song: Option<Vec<SongEntry>>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SongEntry {
    id: String,
    #[serde(default)]
    title: String,
    #[serde(default)]
    artist: String,
    #[serde(default, rename = "artistId")]
    artist_id: String,
    #[serde(default)]
    album: String,
    #[serde(default, rename = "albumId")]
    album_id: String,
    #[serde(rename = "coverArt")]
    cover_art: Option<String>,
    #[serde(default)]
    duration: u32,
    track: Option<u32>,
    year: Option<u32>,
}

impl SongEntry {
    fn into_song(self) -> NavidromeSong {
        NavidromeSong {
            id: self.id,
            title: self.title,
            artist: self.artist,
            artist_id: self.artist_id,
            album: self.album,
            album_id: self.album_id,
            cover_art: self.cover_art,
            duration: self.duration,
            track: self.track,
            year: self.year,
        }
    }
}

#[derive(Debug, Deserialize)]
struct ArtistsWrapper {
    index: Option<Vec<ArtistIndexEntry>>,
}

#[derive(Debug, Deserialize)]
struct ArtistIndexEntry {
    artist: Option<Vec<ArtistEntry2>>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ArtistEntry2 {
    id: String,
    name: String,
    #[serde(rename = "coverArt")]
    cover_art: Option<String>,
    #[serde(default, rename = "albumCount")]
    album_count: u32,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ArtistDetailBody {
    id: String,
    name: String,
    #[serde(rename = "coverArt")]
    cover_art: Option<String>,
    album: Option<Vec<AlbumEntry>>,
}

#[derive(Debug, Deserialize)]
struct GenresWrapper {
    genre: Option<Vec<GenreEntry>>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct GenreEntry {
    value: String,
    #[serde(default, rename = "songCount")]
    song_count: u32,
    #[serde(default, rename = "albumCount")]
    album_count: u32,
}

#[derive(Debug, Deserialize)]
struct SongsByGenreWrapper {
    song: Option<Vec<SongEntry>>,
}

#[derive(Debug, Deserialize)]
struct PlaylistsWrapper {
    playlist: Option<Vec<PlaylistEntry>>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct PlaylistEntry {
    id: String,
    name: String,
    comment: Option<String>,
    #[serde(default, rename = "songCount")]
    song_count: u32,
    #[serde(rename = "coverArt")]
    cover_art: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct PlaylistDetailBody {
    id: String,
    name: String,
    comment: Option<String>,
    #[serde(default, rename = "songCount")]
    song_count: u32,
    #[serde(rename = "coverArt")]
    cover_art: Option<String>,
    entry: Option<Vec<SongEntry>>,
}

// ── Auth helpers ──────────────────────────────────────────────────────────────

/// Compute the Subsonic token: md5(password + salt).
pub fn compute_token(password: &str, salt: &str) -> String {
    let digest = md5::compute(format!("{}{}", password, salt));
    format!("{:x}", digest)
}

/// Build the Subsonic auth query string fragment.
fn auth_params(user: &str, token: &str, salt: &str) -> String {
    format!(
        "u={}&t={}&s={}&v={}&c={}&f=json",
        user, token, salt, API_VERSION, CLIENT_NAME
    )
}

/// Verify credentials by calling /rest/ping.view.
/// Returns `true` if the server responds with `status: "ok"`.
pub async fn ping(base_url: &str, user: &str, token: &str, salt: &str) -> bool {
    let url = format!(
        "{}/rest/ping.view?{}",
        base_url.trim_end_matches('/'),
        auth_params(user, token, salt)
    );
    let client = match reqwest::Client::builder()
        .timeout(Duration::from_secs(10))
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
            match resp.json::<SubsonicEnvelope>().await {
                Ok(env) => env.response.status == "ok",
                Err(e) => {
                    tracing::warn!("Navidrome ping {url}: failed to parse response: {e}");
                    false
                }
            }
        }
        Err(e) => {
            tracing::warn!("Navidrome ping {url}: request failed: {e}");
            false
        }
    }
}

/// List music library root folders via /rest/getMusicFolders.view.
pub async fn list_music_folders(
    base_url: &str,
    user: &str,
    token: &str,
    salt: &str,
) -> Vec<NavidromeEntry> {
    let url = format!(
        "{}/rest/getMusicFolders.view?{}",
        base_url.trim_end_matches('/'),
        auth_params(user, token, salt)
    );
    match fetch_subsonic(&url).await {
        Ok(body) => {
            let folders = body
                .music_folders
                .map(|f| f.music_folder)
                .unwrap_or_default();
            folders
                .into_iter()
                .map(|f| {
                    let id = f.id_str();
                    NavidromeEntry {
                        id: id.clone(),
                        name: f.name.unwrap_or(id),
                        is_container: true,
                        stream_url: None,
                    }
                })
                .collect()
        }
        Err(e) => {
            tracing::warn!("Navidrome getMusicFolders {url}: {e}");
            vec![]
        }
    }
}

/// Browse artists via /rest/getIndexes.view (optionally scoped to a music folder).
pub async fn list_indexes(
    base_url: &str,
    user: &str,
    token: &str,
    salt: &str,
    music_folder_id: Option<&str>,
) -> Vec<NavidromeEntry> {
    let mut url = format!(
        "{}/rest/getIndexes.view?{}",
        base_url.trim_end_matches('/'),
        auth_params(user, token, salt)
    );
    if let Some(fid) = music_folder_id {
        url.push_str(&format!("&musicFolderId={}", fid));
    }
    match fetch_subsonic(&url).await {
        Ok(body) => {
            let letters = body.indexes.and_then(|i| i.index).unwrap_or_default();
            let mut entries = vec![];
            for letter in letters {
                for artist in letter.artist.unwrap_or_default() {
                    entries.push(NavidromeEntry {
                        id: artist.id,
                        name: artist.name,
                        is_container: true,
                        stream_url: None,
                    });
                }
            }
            entries
        }
        Err(e) => {
            tracing::warn!("Navidrome getIndexes {url}: {e}");
            vec![]
        }
    }
}

/// Browse a directory via /rest/getMusicDirectory.view?id=X.
pub async fn browse_directory(
    base_url: &str,
    user: &str,
    token: &str,
    salt: &str,
    dir_id: &str,
) -> Vec<NavidromeEntry> {
    let url = format!(
        "{}/rest/getMusicDirectory.view?id={}&{}",
        base_url.trim_end_matches('/'),
        dir_id,
        auth_params(user, token, salt)
    );
    match fetch_subsonic(&url).await {
        Ok(body) => {
            let children = body.directory.and_then(|d| d.child).unwrap_or_default();
            children
                .into_iter()
                .map(|c| {
                    let is_dir = c.is_dir.unwrap_or(false);
                    let name = c.display_name();
                    let stream_url = if !is_dir {
                        Some(format!(
                            "{}/rest/stream.view?id={}&{}",
                            base_url.trim_end_matches('/'),
                            c.id,
                            auth_params(user, token, salt)
                        ))
                    } else {
                        None
                    };
                    NavidromeEntry {
                        id: c.id,
                        name,
                        is_container: is_dir,
                        stream_url,
                    }
                })
                .collect()
        }
        Err(e) => {
            tracing::warn!("Navidrome getMusicDirectory {url}: {e}");
            vec![]
        }
    }
}

async fn fetch_subsonic(url: &str) -> Result<SubsonicBody, anyhow::Error> {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(10))
        .build()?;
    let resp = client.get(url).send().await?;
    let status = resp.status();
    let env: SubsonicEnvelope = resp.json().await?;
    if !status.is_success() {
        anyhow::bail!("HTTP {status}");
    }
    if env.response.status != "ok" {
        anyhow::bail!("Subsonic status: {}", env.response.status);
    }
    Ok(env.response)
}

/// Split `http://host:port?nd_user=u&nd_token=t&nd_salt=s` into its components.
/// Returns `(base_url, Option<user>, Option<token>, Option<salt>)`.
pub fn parse_base_url(url: &str) -> (String, Option<String>, Option<String>, Option<String>) {
    if let Some(idx) = url.find('?') {
        let base = url[..idx].to_string();
        let query = &url[idx + 1..];
        let get = |key: &str| -> Option<String> {
            query
                .split('&')
                .find(|p| p.starts_with(key))
                .and_then(|p| p.strip_prefix(key))
                .filter(|v| !v.is_empty())
                .map(|v| v.to_string())
        };
        (base, get("nd_user="), get("nd_token="), get("nd_salt="))
    } else {
        (url.to_string(), None, None, None)
    }
}

pub fn percent_encode(s: &str) -> String {
    let mut out = String::with_capacity(s.len() * 3 / 2);
    for b in s.bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(b as char);
            }
            _ => {
                out.push('%');
                out.push(char::from_digit((b >> 4) as u32, 16).unwrap());
                out.push(char::from_digit((b & 0xf) as u32, 16).unwrap());
            }
        }
    }
    out
}

pub fn percent_decode(s: &str) -> String {
    let mut out = Vec::with_capacity(s.len());
    let bytes = s.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' && i + 2 < bytes.len() {
            if let (Some(hi), Some(lo)) = (
                (bytes[i + 1] as char).to_digit(16),
                (bytes[i + 2] as char).to_digit(16),
            ) {
                out.push(((hi << 4) | lo) as u8);
                i += 3;
                continue;
            }
        }
        out.push(bytes[i]);
        i += 1;
    }
    String::from_utf8_lossy(&out).into_owned()
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

// ── Cover art and stream URL helpers ─────────────────────────────────────────

pub fn cover_art_url(
    base_url: &str,
    user: &str,
    token: &str,
    salt: &str,
    id: &str,
    size: Option<u32>,
) -> String {
    let size_param = size.map(|s| format!("&size={s}")).unwrap_or_default();
    format!(
        "{}/rest/getCoverArt.view?id={}{}&{}",
        base_url.trim_end_matches('/'),
        id,
        size_param,
        auth_params(user, token, salt)
    )
}

pub fn stream_url(base_url: &str, user: &str, token: &str, salt: &str, song_id: &str) -> String {
    format!(
        "{}/rest/stream.view?id={}&{}",
        base_url.trim_end_matches('/'),
        song_id,
        auth_params(user, token, salt)
    )
}

// ── Rich client methods ───────────────────────────────────────────────────────

/// List albums alphabetically (or by another list_type).
/// `list_type` values: "alphabeticalByName", "alphabeticalByArtist", "newest",
/// "highest", "frequent", "recent", "random".
pub async fn get_albums(
    base_url: &str,
    user: &str,
    token: &str,
    salt: &str,
    list_type: &str,
    size: usize,
    offset: usize,
) -> Vec<NavidromeAlbum> {
    let url = format!(
        "{}/rest/getAlbumList2.view?type={}&size={}&offset={}&{}",
        base_url.trim_end_matches('/'),
        list_type,
        size,
        offset,
        auth_params(user, token, salt)
    );
    match fetch_subsonic(&url).await {
        Ok(body) => body
            .album_list2
            .and_then(|w| w.album)
            .unwrap_or_default()
            .into_iter()
            .map(|a| NavidromeAlbum {
                id: a.id,
                name: a.name,
                artist: a.artist,
                artist_id: a.artist_id,
                year: a.year,
                cover_art: a.cover_art,
                song_count: a.song_count,
                duration: a.duration,
            })
            .collect(),
        Err(e) => {
            tracing::warn!("Navidrome getAlbumList2 {url}: {e}");
            vec![]
        }
    }
}

/// Get a single album with its tracks.
pub async fn get_album_with_songs(
    base_url: &str,
    user: &str,
    token: &str,
    salt: &str,
    album_id: &str,
) -> Option<(NavidromeAlbum, Vec<NavidromeSong>)> {
    let url = format!(
        "{}/rest/getAlbum.view?id={}&{}",
        base_url.trim_end_matches('/'),
        album_id,
        auth_params(user, token, salt)
    );
    match fetch_subsonic(&url).await {
        Ok(body) => {
            let a = body.album?;
            let album = NavidromeAlbum {
                id: a.id,
                name: a.name,
                artist: a.artist,
                artist_id: a.artist_id,
                year: a.year,
                cover_art: a.cover_art.clone(),
                song_count: a.song_count,
                duration: a.duration,
            };
            let songs = a
                .song
                .unwrap_or_default()
                .into_iter()
                .map(|s| s.into_song())
                .collect();
            Some((album, songs))
        }
        Err(e) => {
            tracing::warn!("Navidrome getAlbum {url}: {e}");
            None
        }
    }
}

/// Fetch a single song's metadata (used to resolve cover art for the currently playing track).
pub async fn get_song(
    base_url: &str,
    user: &str,
    token: &str,
    salt: &str,
    song_id: &str,
) -> Option<NavidromeSong> {
    let url = format!(
        "{}/rest/getSong.view?id={}&{}",
        base_url.trim_end_matches('/'),
        song_id,
        auth_params(user, token, salt)
    );
    match fetch_subsonic(&url).await {
        Ok(body) => body.song.map(|s| s.into_song()),
        Err(e) => {
            tracing::warn!("Navidrome getSong {url}: {e}");
            None
        }
    }
}

/// List all artists.
pub async fn get_artists(
    base_url: &str,
    user: &str,
    token: &str,
    salt: &str,
) -> Vec<NavidromeArtist> {
    let url = format!(
        "{}/rest/getArtists.view?{}",
        base_url.trim_end_matches('/'),
        auth_params(user, token, salt)
    );
    match fetch_subsonic(&url).await {
        Ok(body) => {
            let indices = body.artists.and_then(|a| a.index).unwrap_or_default();
            let mut artists = vec![];
            for idx in indices {
                for a in idx.artist.unwrap_or_default() {
                    artists.push(NavidromeArtist {
                        id: a.id,
                        name: a.name,
                        cover_art: a.cover_art,
                        album_count: a.album_count,
                    });
                }
            }
            artists
        }
        Err(e) => {
            tracing::warn!("Navidrome getArtists {url}: {e}");
            vec![]
        }
    }
}

/// Get a single artist with their albums.
pub async fn get_artist_with_albums(
    base_url: &str,
    user: &str,
    token: &str,
    salt: &str,
    artist_id: &str,
) -> Option<(NavidromeArtist, Vec<NavidromeAlbum>)> {
    let url = format!(
        "{}/rest/getArtist.view?id={}&{}",
        base_url.trim_end_matches('/'),
        artist_id,
        auth_params(user, token, salt)
    );
    match fetch_subsonic(&url).await {
        Ok(body) => {
            let a = body.artist?;
            let artist = NavidromeArtist {
                id: a.id,
                name: a.name.clone(),
                cover_art: a.cover_art,
                album_count: a.album.as_ref().map(|v| v.len() as u32).unwrap_or(0),
            };
            let albums = a
                .album
                .unwrap_or_default()
                .into_iter()
                .map(|al| NavidromeAlbum {
                    id: al.id,
                    name: al.name,
                    artist: al.artist,
                    artist_id: al.artist_id,
                    year: al.year,
                    cover_art: al.cover_art,
                    song_count: al.song_count,
                    duration: al.duration,
                })
                .collect();
            Some((artist, albums))
        }
        Err(e) => {
            tracing::warn!("Navidrome getArtist {url}: {e}");
            None
        }
    }
}

/// List all genres.
pub async fn get_genres(
    base_url: &str,
    user: &str,
    token: &str,
    salt: &str,
) -> Vec<NavidromeGenre> {
    let url = format!(
        "{}/rest/getGenres.view?{}",
        base_url.trim_end_matches('/'),
        auth_params(user, token, salt)
    );
    match fetch_subsonic(&url).await {
        Ok(body) => body
            .genres
            .and_then(|g| g.genre)
            .unwrap_or_default()
            .into_iter()
            .map(|g| NavidromeGenre {
                name: g.value,
                song_count: g.song_count,
                album_count: g.album_count,
            })
            .collect(),
        Err(e) => {
            tracing::warn!("Navidrome getGenres {url}: {e}");
            vec![]
        }
    }
}

/// Get songs for a genre (up to `size` starting at `offset`).
pub async fn get_songs_by_genre(
    base_url: &str,
    user: &str,
    token: &str,
    salt: &str,
    genre: &str,
    size: usize,
    offset: usize,
) -> Vec<NavidromeSong> {
    let encoded_genre = percent_encode(genre);
    let url = format!(
        "{}/rest/getSongsByGenre.view?genre={}&count={}&offset={}&{}",
        base_url.trim_end_matches('/'),
        encoded_genre,
        size,
        offset,
        auth_params(user, token, salt)
    );
    match fetch_subsonic(&url).await {
        Ok(body) => body
            .songs_by_genre
            .and_then(|w| w.song)
            .unwrap_or_default()
            .into_iter()
            .map(|s| s.into_song())
            .collect(),
        Err(e) => {
            tracing::warn!("Navidrome getSongsByGenre {url}: {e}");
            vec![]
        }
    }
}

/// List all playlists.
pub async fn get_playlists(
    base_url: &str,
    user: &str,
    token: &str,
    salt: &str,
) -> Vec<NavidromePlaylist> {
    let url = format!(
        "{}/rest/getPlaylists.view?{}",
        base_url.trim_end_matches('/'),
        auth_params(user, token, salt)
    );
    match fetch_subsonic(&url).await {
        Ok(body) => body
            .playlists
            .and_then(|w| w.playlist)
            .unwrap_or_default()
            .into_iter()
            .map(|p| NavidromePlaylist {
                id: p.id,
                name: p.name,
                comment: p.comment,
                song_count: p.song_count,
                cover_art: p.cover_art,
            })
            .collect(),
        Err(e) => {
            tracing::warn!("Navidrome getPlaylists {url}: {e}");
            vec![]
        }
    }
}

/// Get a single playlist with its tracks.
pub async fn get_playlist_with_tracks(
    base_url: &str,
    user: &str,
    token: &str,
    salt: &str,
    playlist_id: &str,
) -> Option<(NavidromePlaylist, Vec<NavidromeSong>)> {
    let url = format!(
        "{}/rest/getPlaylist.view?id={}&{}",
        base_url.trim_end_matches('/'),
        playlist_id,
        auth_params(user, token, salt)
    );
    match fetch_subsonic(&url).await {
        Ok(body) => {
            let p = body.playlist?;
            let playlist = NavidromePlaylist {
                id: p.id,
                name: p.name,
                comment: p.comment,
                song_count: p.song_count,
                cover_art: p.cover_art,
            };
            let tracks = p
                .entry
                .unwrap_or_default()
                .into_iter()
                .map(|s| s.into_song())
                .collect();
            Some((playlist, tracks))
        }
        Err(e) => {
            tracing::warn!("Navidrome getPlaylist {url}: {e}");
            None
        }
    }
}

// ── Starring ──────────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
struct Starred2Wrapper {
    song: Option<Vec<SongEntry>>,
}

#[derive(Debug, Deserialize)]
struct SearchResult3 {
    song: Option<Vec<SongEntry>>,
}

/// Star (like) a song.
pub async fn star_song(base_url: &str, user: &str, token: &str, salt: &str, id: &str) -> bool {
    let url = format!(
        "{}/rest/star.view?id={}&{}",
        base_url.trim_end_matches('/'),
        id,
        auth_params(user, token, salt)
    );
    match fetch_subsonic(&url).await {
        Ok(_) => true,
        Err(e) => {
            tracing::warn!("Navidrome star {url}: {e}");
            false
        }
    }
}

/// Unstar (unlike) a song.
pub async fn unstar_song(base_url: &str, user: &str, token: &str, salt: &str, id: &str) -> bool {
    let url = format!(
        "{}/rest/unstar.view?id={}&{}",
        base_url.trim_end_matches('/'),
        id,
        auth_params(user, token, salt)
    );
    match fetch_subsonic(&url).await {
        Ok(_) => true,
        Err(e) => {
            tracing::warn!("Navidrome unstar {url}: {e}");
            false
        }
    }
}

/// Scrobble a song play to Last.fm via the Subsonic API.
/// `submission=true` → record the play; `submission=false` → "now playing" notification.
pub async fn scrobble_song(base_url: &str, user: &str, token: &str, salt: &str, id: &str) -> bool {
    let url = format!(
        "{}/rest/scrobble.view?id={}&submission=true&{}",
        base_url.trim_end_matches('/'),
        id,
        auth_params(user, token, salt)
    );
    match fetch_subsonic(&url).await {
        Ok(_) => true,
        Err(e) => {
            tracing::warn!("Navidrome scrobble {url}: {e}");
            false
        }
    }
}

/// Get all starred (liked) songs.
pub async fn get_starred(
    base_url: &str,
    user: &str,
    token: &str,
    salt: &str,
) -> Vec<NavidromeSong> {
    let url = format!(
        "{}/rest/getStarred2.view?{}",
        base_url.trim_end_matches('/'),
        auth_params(user, token, salt)
    );
    match fetch_subsonic(&url).await {
        Ok(body) => body
            .starred2
            .and_then(|s| s.song)
            .unwrap_or_default()
            .into_iter()
            .map(|s| s.into_song())
            .collect(),
        Err(e) => {
            tracing::warn!("Navidrome getStarred2 {url}: {e}");
            vec![]
        }
    }
}

/// Search songs by query (up to `count` results, empty query = all songs).
pub async fn search_songs(
    base_url: &str,
    user: &str,
    token: &str,
    salt: &str,
    query: &str,
    count: u32,
) -> Vec<NavidromeSong> {
    let url = format!(
        "{}/rest/search3.view?query={}&songCount={}&albumCount=0&artistCount=0&{}",
        base_url.trim_end_matches('/'),
        percent_encode(query),
        count,
        auth_params(user, token, salt)
    );
    match fetch_subsonic(&url).await {
        Ok(body) => body
            .search_result3
            .and_then(|s| s.song)
            .unwrap_or_default()
            .into_iter()
            .map(|s| s.into_song())
            .collect(),
        Err(e) => {
            tracing::warn!("Navidrome search3 {url}: {e}");
            vec![]
        }
    }
}
