use std::{collections::HashMap, fs};

use anyhow::Error;
use mpd_filters::{Expression, Parser, SqlOptions, ToSql};
use regex::Regex;
use rockbox_library::{entity::track::Track, repo};
use rockbox_rpc::api::rockbox::v1alpha1::{
    GetAlbumsRequest, GetArtistsRequest, GetGlobalSettingsRequest, GetTracksRequest,
    ScanLibraryRequest, SearchRequest,
};
use rockbox_settings::get_music_dir;
use tokio::sync::mpsc::Sender;

use crate::Context;

pub async fn handle_list_album(
    ctx: &mut Context,
    request: &str,
    tx: Sender<String>,
) -> Result<String, Error> {
    let query = request.replace("list album", "").replace("list Album", "");
    let query = query.trim();
    let query = query.trim_matches('"');
    let query = query.replace(r#"\\"#, r#"\"#);
    let mut albums = repo::album::all(ctx.pool.clone()).await?;

    if !query.is_empty() {
        let mut columns = HashMap::new();
        columns.insert("AlbumArtist".to_string(), "artist".to_string());
        let opts = SqlOptions {
            columns,
            ..Default::default()
        };
        let mut parser = Parser::new(&query);
        let expr = parser.parse().map_err(|e| Error::msg(e))?;
        albums = repo::album::filter(ctx.pool.clone(), expr.to_sql(opts)).await?;
    }

    let response = albums
        .iter()
        .map(|x| format!("Album: {}\n", x.title))
        .collect::<String>();
    let response = format!("{}OK\n", response);

    if !ctx.batch {
        tx.send(response.clone()).await?;
    }

    Ok(response)
}

pub async fn handle_list_artist(
    ctx: &mut Context,
    _request: &str,
    tx: Sender<String>,
) -> Result<String, Error> {
    let response = ctx.library.get_artists(GetArtistsRequest {}).await?;
    let response = response.into_inner();
    let response = response
        .artists
        .iter()
        .map(|x| format!("Artist: {}\n", x.name))
        .collect::<String>();
    let response = format!("{}OK\n", response);
    if !ctx.batch {
        tx.send(response.clone()).await?;
    }
    Ok(response)
}

pub async fn handle_list_title(
    ctx: &mut Context,
    _request: &str,
    tx: Sender<String>,
) -> Result<String, Error> {
    let response = ctx.library.get_tracks(GetTracksRequest {}).await?;
    let response = response.into_inner();
    let response = response
        .tracks
        .iter()
        .map(|x| format!("Title: {}\n", x.title))
        .collect::<String>();
    let response = format!("{}OK\n", response);
    if !ctx.batch {
        tx.send(response.clone()).await?;
    }
    Ok(response)
}

pub async fn handle_search(
    ctx: &mut Context,
    request: &str,
    tx: Sender<String>,
) -> Result<String, Error> {
    let mut term = request
        .trim_matches('"')
        .replace("search Album", "")
        .replace("search Artist", "")
        .replace("search Title", "")
        .replace("search album", "")
        .replace("search artist", "")
        .replace("search title", "")
        .trim()
        .to_string();

    if term.starts_with("search ") {
        let query = &term[7..];
        term = query.to_string();
        term = term.trim().to_string();
        term = term.trim_matches('"').to_string();
    }

    let mut parser = Parser::new(&term);
    if let Ok(expr) = parser.parse() {
        let response = evaluate_search_expression(ctx, &expr, false)
            .await?
            .iter()
            .map(|x| {
                format!(
                "file: {}\nArtist: {}\nAlbum: {}\nTitle: {}\nTrack: {}\nTime: {}\nDuration: {}\n",
                x.path,
                x.artist,
                x.album,
                x.title,
                x.track_number.unwrap_or_default(),
                (x.length / 1000) as u32,
                x.length / 1000
            )
            })
            .collect::<String>();
        let response = format!("{}OK\n", response);
        if !ctx.batch {
            tx.send(response.clone()).await?;
        }
        return Ok(response);
    }

    let response = ctx.library.search(SearchRequest { term }).await?;
    let response = response.into_inner();

    let response = response
        .tracks
        .iter()
        .map(|x| {
            format!(
                "file: {}\nArtist: {}\nAlbum: {}\nTitle: {}\nTrack: {}\nTime: {}\nDuration: {}\n",
                x.path,
                x.artist,
                x.album,
                x.title,
                x.track_number,
                (x.length / 1000) as u32,
                x.length / 1000
            )
        })
        .collect::<String>();
    let response = format!("{}OK\n", response);
    if !ctx.batch {
        tx.send(response.clone()).await?;
    }
    Ok(response)
}

pub async fn handle_rescan(
    ctx: &mut Context,
    request: &str,
    tx: Sender<String>,
) -> Result<String, Error> {
    let response = ctx
        .settings
        .get_global_settings(GetGlobalSettingsRequest {})
        .await?;
    let response = response.into_inner();
    let path = request
        .replace("update ", "")
        .replace("rescan ", "")
        .trim_matches('"')
        .to_string();
    let path = Some(match path.starts_with("/") {
        true => path,
        false => format!("{}/{}", response.music_dir, path),
    });
    ctx.library
        .scan_library(ScanLibraryRequest {
            path,
            rebuild_index: None,
        })
        .await?;

    if !ctx.batch {
        tx.send("OK\n".to_string()).await?;
    }
    Ok("OK\n".to_string())
}

pub async fn handle_config(
    ctx: &mut Context,
    _request: &str,
    tx: Sender<String>,
) -> Result<String, Error> {
    let response = "ACK [4@0] {config} Command only permitted to local clients";
    if !ctx.batch {
        tx.send(response.to_string()).await?;
    }

    Ok(response.to_string())
}

pub async fn handle_tagtypes(
    ctx: &mut Context,
    _request: &str,
    tx: Sender<String>,
) -> Result<String, Error> {
    let response = format!(
        "Tagtype: Artist\nTagtype: Album\nTagtype: Title\nTagtype: Track\nTagtype: Date\nOK\n"
    );

    if !ctx.batch {
        tx.send(response.clone()).await?;
    }

    Ok(response)
}

pub async fn handle_tagtypes_clear(
    ctx: &mut Context,
    _request: &str,
    tx: Sender<String>,
) -> Result<String, Error> {
    let response = format!("OK\n");

    if !ctx.batch {
        tx.send(response.clone()).await?;
    }

    Ok(response)
}

pub async fn handle_tagtypes_enable(
    ctx: &mut Context,
    _request: &str,
    tx: Sender<String>,
) -> Result<String, Error> {
    let response = format!("OK\n");

    if !ctx.batch {
        tx.send(response.to_string()).await?;
    }

    Ok("".to_string())
}

pub async fn handle_stats(
    ctx: &mut Context,
    _request: &str,
    tx: Sender<String>,
) -> Result<String, Error> {
    let response = ctx.library.get_albums(GetAlbumsRequest {}).await?;
    let response = response.into_inner();
    let albums = response.albums.len();
    let response = ctx.library.get_artists(GetArtistsRequest {}).await?;
    let response = response.into_inner();
    let artists = response.artists.len();
    let response = ctx.library.get_tracks(GetTracksRequest {}).await?;
    let response = response.into_inner();
    let tracks = response.tracks.len();
    let response = format!(
        "artists: {}\nalbums: {}\nsongs: {}\nOK\n",
        artists, albums, tracks
    );

    if !ctx.batch {
        tx.send(response.clone()).await?;
    }

    Ok(response)
}

pub async fn handle_find_artist(
    ctx: &mut Context,
    request: &str,
    tx: Sender<String>,
) -> Result<String, Error> {
    let re = Regex::new(r#"(?i)(artist|album|date)\s+\"([^\"]+)\""#).unwrap();
    let mut fields = HashMap::new();

    for caps in re.captures_iter(request) {
        let key = caps.get(1).map(|m| m.as_str()).unwrap().to_lowercase();
        let value = caps.get(2).map(|m| m.as_str()).unwrap();
        fields.insert(key, value);
    }
    let artist = fields.get("artist");
    let album = fields.get("album");
    let date = fields.get("date");
    if artist.is_none() {
        return Ok("ACK [2@0] {find} missing \"artist\" argument\n".to_string());
    }

    let artist = *artist.unwrap();
    let tracks = match (album, date) {
        (Some(album), Some(date)) => {
            repo::track::find_by_artist_album_date(ctx.pool.clone(), artist, *album, *date).await?
        }
        _ => repo::track::find_by_artist(ctx.pool.clone(), artist).await?,
    };

    let mut response: String = "".to_string();

    build_file_metadata(tracks, &mut response).await?;

    if !ctx.batch {
        tx.send(response.clone()).await?;
    }

    Ok(response)
}

pub async fn handle_find_album(
    ctx: &mut Context,
    request: &str,
    tx: Sender<String>,
) -> Result<String, Error> {
    let arg = request.replace("find album ", "").replace("find Album", "");
    let arg = arg.trim();
    let arg = arg.trim_matches('"');
    let tracks = repo::track::find_by_album(ctx.pool.clone(), arg).await?;

    let mut response: String = "".to_string();

    build_file_metadata(tracks, &mut response).await?;

    if !ctx.batch {
        tx.send(response.clone()).await?;
    }

    Ok(response)
}

pub async fn handle_find_title(
    ctx: &mut Context,
    request: &str,
    tx: Sender<String>,
) -> Result<String, Error> {
    let arg = request
        .replace("find title ", "")
        .replace("find Title ", "");
    let arg = arg.trim();
    let arg = arg.trim_matches('"');
    let tracks = repo::track::find_by_title(ctx.pool.clone(), arg).await?;

    let mut response: String = "".to_string();

    build_file_metadata(tracks, &mut response).await?;

    if !ctx.batch {
        tx.send(response.clone()).await?;
    }

    Ok(response)
}

pub async fn handle_find(
    ctx: &mut Context,
    request: &str,
    tx: Sender<String>,
) -> Result<String, Error> {
    let arg = request.replace("find ", "");
    let arg = arg.trim();
    let arg = arg.trim_matches('"');
    let arg = arg.replace(r#"\\"#, r#"\"#);
    let mut parser = Parser::new(&arg);
    match parser.parse() {
        Ok(expr) => {
            execute(ctx, &expr, tx).await?;
        }
        Err(e) => return Err(Error::msg(e)),
    }
    Ok("".to_string())
}

async fn execute(ctx: &mut Context, expr: &Expression, tx: Sender<String>) -> Result<(), Error> {
    let mut columns = HashMap::new();
    columns.insert("Title".to_string(), "title".to_string());
    columns.insert("Artist".to_string(), "artist".to_string());
    columns.insert("Album".to_string(), "album".to_string());
    columns.insert("AlbumArtist".to_string(), "album_artist".to_string());
    columns.insert("File".to_string(), "path".to_string());

    let opts = SqlOptions {
        columns,
        file_prefix: Some(match get_music_dir()?.ends_with("/") {
            true => get_music_dir()?,
            false => format!("{}/", get_music_dir()?),
        }),
    };

    let query = expr.to_sql(opts);

    let tracks = match !query.0.contains(" REGEXP") {
        true => repo::track::filter(ctx.pool.clone(), query).await?,
        false => evaluate_search_expression(ctx, expr, true).await?,
    };

    let mut response: String = "".to_string();

    build_file_metadata(tracks, &mut response).await?;
    tx.send(response).await?;
    Ok(())
}

async fn evaluate_search_expression(
    ctx: &mut Context,
    expr: &Expression,
    case_sensitive: bool,
) -> Result<Vec<Track>, Error> {
    let mut tracks = repo::track::all(ctx.pool.clone()).await?;
    tracks = tracks
        .into_iter()
        .filter(|track| {
            let mut record = HashMap::new();
            record.insert("title".to_string(), track.title.clone());
            record.insert("artist".to_string(), track.artist.clone());
            record.insert("album".to_string(), track.album.clone());
            record.insert("album_artist".to_string(), track.album_artist.clone());
            record.insert("file".to_string(), track.path.clone());
            record.insert("filename".to_string(), track.path.clone());
            record.insert(
                "genre".to_string(),
                track.genre.clone().unwrap_or("".to_string()),
            );
            record.insert("Title".to_string(), track.title.clone());
            record.insert("Artist".to_string(), track.artist.clone());
            record.insert("Album".to_string(), track.album.clone());
            record.insert("AlbumArtist".to_string(), track.album_artist.clone());
            record.insert("File".to_string(), track.path.clone());
            record.insert("Filename".to_string(), track.path.clone());
            record.insert(
                "Genre".to_string(),
                track.genre.clone().unwrap_or("".to_string()),
            );
            expr.evaluate(&record, case_sensitive)
        })
        .collect();
    Ok(tracks)
}

async fn build_file_metadata(tracks: Vec<Track>, response: &mut String) -> Result<(), Error> {
    let music_dir = get_music_dir()?;

    for track in tracks {
        let file = track.path.replace(&music_dir, "");
        let file = file.chars().skip(1).collect::<String>();

        let last_modified = fs::metadata(track.path)?.modified().unwrap();
        let last_modified = chrono::DateTime::from_timestamp(
            last_modified
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs() as i64,
            0,
        )
        .unwrap();
        let last_modified = last_modified.format("%Y-%m-%dT%H:%M:%SZ").to_string();
        response.push_str(&format!(
            "file: {}\nLast-Modified: {}\n",
            file, last_modified
        ));

        response.push_str(&format!(
            "Title: {}\nArtist: {}\nAlbum: {}\nTime: {}\nDuration: {}\nAlbumArtist: {}\n",
            track.title,
            track.artist,
            track.album,
            (track.length / 1000) as u32,
            track.length / 1000,
            track.album_artist,
        ));
        if let Some(track_number) = track.track_number {
            response.push_str(&format!("Track: {}\n", track_number));
        }

        if let Some(year_string) = track.year_string {
            response.push_str(&format!("Date: {}\n", year_string));
        }
    }

    response.push_str("OK\n");
    Ok(())
}
