use anyhow::Error;
use rockbox_library::repo;
use rockbox_rpc::api::rockbox::v1alpha1::{
    GetAlbumsRequest, GetArtistsRequest, GetGlobalSettingsRequest, GetTracksRequest,
    ScanLibraryRequest, SearchRequest,
};
use tokio::{
    io::{AsyncWriteExt, BufReader},
    net::TcpStream,
};

use crate::Context;

pub async fn handle_list_album(
    ctx: &mut Context,
    _request: &str,
    stream: &mut BufReader<TcpStream>,
) -> Result<String, Error> {
    let response = ctx.library.get_albums(GetAlbumsRequest {}).await?;
    let response = response.into_inner();
    let response = response
        .albums
        .iter()
        .map(|x| format!("Album: {}\n", x.title))
        .collect::<String>();
    let response = format!("{}OK\n", response);

    if !ctx.batch {
        stream.write_all(response.as_bytes()).await?;
    }

    Ok(response)
}

pub async fn handle_list_artist(
    ctx: &mut Context,
    _request: &str,
    stream: &mut BufReader<TcpStream>,
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
        stream.write_all(response.as_bytes()).await?;
    }
    Ok(response)
}

pub async fn handle_list_title(
    ctx: &mut Context,
    _request: &str,
    stream: &mut BufReader<TcpStream>,
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
        stream.write_all(response.as_bytes()).await?;
    }
    Ok(response)
}

pub async fn handle_search(
    ctx: &mut Context,
    request: &str,
    stream: &mut BufReader<TcpStream>,
) -> Result<String, Error> {
    let term = request
        .replace("\"", "")
        .replace("search Album", "")
        .replace("search Artist", "")
        .replace("search Title", "")
        .replace("search album", "")
        .replace("search artist", "")
        .replace("search title", "")
        .trim()
        .to_string();
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
        stream.write_all(response.as_bytes()).await?;
    }
    Ok(response)
}

pub async fn handle_rescan(
    ctx: &mut Context,
    request: &str,
    stream: &mut BufReader<TcpStream>,
) -> Result<String, Error> {
    let response = ctx
        .settings
        .get_global_settings(GetGlobalSettingsRequest {})
        .await?;
    let response = response.into_inner();
    let path = request
        .replace("update ", "")
        .replace("rescan ", "")
        .replace("\"", "");
    let path = Some(match path.starts_with("/") {
        true => path,
        false => format!("{}/{}", response.music_dir, path),
    });
    ctx.library
        .scan_library(ScanLibraryRequest { path })
        .await?;

    if !ctx.batch {
        stream.write_all(b"OK\n").await?;
    }
    Ok("OK\n".to_string())
}

pub async fn handle_config(
    ctx: &mut Context,
    _request: &str,
    stream: &mut BufReader<TcpStream>,
) -> Result<String, Error> {
    let response = "ACK [4@0] {config} Command only permitted to local clients";
    if !ctx.batch {
        stream.write_all(response.as_bytes()).await?;
    }

    Ok(response.to_string())
}

pub async fn handle_tagtypes(
    ctx: &mut Context,
    _request: &str,
    stream: &mut BufReader<TcpStream>,
) -> Result<String, Error> {
    let response = format!(
        "Tagtype: Artist\nTagtype: Album\nTagtype: Title\nTagtype: Track\nTagtype: Date\nOK\n"
    );

    if !ctx.batch {
        stream.write_all(response.as_bytes()).await?;
    }

    Ok(response)
}

pub async fn handle_tagtypes_clear(
    ctx: &mut Context,
    _request: &str,
    stream: &mut BufReader<TcpStream>,
) -> Result<String, Error> {
    let response = format!("OK\n");

    if !ctx.batch {
        stream.write_all(response.as_bytes()).await?;
    }

    Ok(response)
}

pub async fn handle_tagtypes_enable(
    ctx: &mut Context,
    _request: &str,
    stream: &mut BufReader<TcpStream>,
) -> Result<String, Error> {
    let response = format!("OK\n");

    if !ctx.batch {
        stream.write_all(response.as_bytes()).await?;
    }

    Ok("".to_string())
}

pub async fn handle_stats(
    ctx: &mut Context,
    _request: &str,
    stream: &mut BufReader<TcpStream>,
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
        stream.write_all(response.as_bytes()).await?;
    }

    Ok(response)
}

pub async fn handle_find_artist(
    ctx: &mut Context,
    request: &str,
    stream: &mut BufReader<TcpStream>,
) -> Result<String, Error> {
    repo::track::find_by_artist(ctx.pool.clone(), "").await?;
    todo!()
}

pub async fn handle_find_album(
    ctx: &mut Context,
    _request: &str,
    stream: &mut BufReader<TcpStream>,
) -> Result<String, Error> {
    repo::track::find_by_album(ctx.pool.clone(), "").await?;
    todo!()
}

pub async fn handle_find_title(
    ctx: &mut Context,
    _request: &str,
    stream: &mut BufReader<TcpStream>,
) -> Result<String, Error> {
    repo::track::find_by_title(ctx.pool.clone(), "").await?;
    todo!()
}
