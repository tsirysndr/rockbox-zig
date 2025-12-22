use std::{env, thread};

use crate::http::{Context, Request, Response};
use anyhow::Error;
use owo_colors::OwoColorize;
use rockbox_library::{artists::update_metadata, audio_scan::scan_audio_files, repo};
use rockbox_search::{
    album::Album, artist::Artist, delete_all_documents, index_entity, liked_album::LikedAlbum,
    liked_track::LikedTrack, track::Track,
};
use rockbox_sys as rb;

pub async fn get_status(_ctx: &Context, _req: &Request, res: &mut Response) -> Result<(), Error> {
    let status = rb::system::get_global_status();
    res.json(&status);
    Ok(())
}

pub async fn get_rockbox_version(
    _ctx: &Context,
    _req: &Request,
    res: &mut Response,
) -> Result<(), Error> {
    let version = rb::system::get_rockbox_version();
    res.json(&version);
    Ok(())
}

pub async fn scan_library(ctx: &Context, req: &Request, res: &mut Response) -> Result<(), Error> {
    let home = env::var("HOME")?;
    let music_library = format!("{}/Music", home);

    let path = match req.query_params.get("path") {
        Some(path) => path.as_str().unwrap_or(&music_library),
        None => &music_library,
    };

    scan_audio_files(ctx.pool.clone(), path.into()).await?;

    let rebuild_index = match req.query_params.get("rebuild_index") {
        Some(rebuild_index) => {
            let rebuild_index = rebuild_index.as_str().unwrap_or("false");
            rebuild_index == "true" || rebuild_index == "1"
        }
        None => false,
    };

    if path != music_library {
        res.text("0");
        return Ok(());
    }

    update_metadata(ctx.pool.clone())?;

    if !rebuild_index {
        res.text("0");
        return Ok(());
    }

    let tracks = repo::track::all(ctx.pool.clone()).await?;
    let albums = repo::album::all(ctx.pool.clone()).await?;
    let artists = repo::artist::all(ctx.pool.clone()).await?;
    let liked_albums = repo::favourites::all_albums(ctx.pool.clone()).await?;
    let liked_tracks = repo::favourites::all_tracks(ctx.pool.clone()).await?;

    let tracks_index = ctx.indexes.tracks.clone();
    let albums_index = ctx.indexes.albums.clone();
    let artists_index = ctx.indexes.artists.clone();
    let liked_albums_index = ctx.indexes.liked_albums.clone();
    let liked_tracks_index = ctx.indexes.liked_tracks.clone();

    thread::spawn(move || {
        match delete_all_documents(&tracks_index) {
            Ok(_) => {}
            Err(e) => eprintln!("Error deleting all documents: {:?}", e),
        }
        let mut i = 1;
        let len = tracks.len();
        for track in tracks {
            println!(
                "[{}/{}] Indexing track: {}",
                i,
                len,
                track.title.bright_green()
            );
            index_entity::<Track>(&tracks_index, &track.into()).unwrap();
            i += 1;
        }
    });

    thread::spawn(move || {
        match delete_all_documents(&albums_index) {
            Ok(_) => {}
            Err(e) => eprintln!("Error deleting all documents: {:?}", e),
        }
        let mut i = 1;
        let len = albums.len();
        for album in albums {
            println!(
                "[{}/{}] Indexing album: {}",
                i,
                len,
                album.title.bright_green()
            );
            index_entity::<Album>(&albums_index, &album.into()).unwrap();
            i += 1;
        }
    });

    thread::spawn(move || {
        match delete_all_documents(&artists_index) {
            Ok(_) => {}
            Err(e) => eprintln!("Error deleting all documents: {:?}", e),
        }
        let mut i = 1;
        let len = artists.len();
        for artist in artists {
            println!(
                "[{}/{}] Indexing artist: {}",
                i,
                len,
                artist.name.bright_green()
            );
            index_entity::<Artist>(&artists_index, &artist.into()).unwrap();
            i += 1;
        }
    });

    thread::spawn(move || {
        match delete_all_documents(&liked_albums_index) {
            Ok(_) => {}
            Err(e) => eprintln!("Error deleting all documents: {:?}", e),
        }
        let mut i = 1;
        let len = liked_albums.len();
        for liked_album in liked_albums {
            println!(
                "[{}/{}] Indexing liked album: {}",
                i,
                len,
                liked_album.title.bright_green()
            );
            index_entity::<LikedAlbum>(&liked_albums_index, &liked_album.into()).unwrap();
            i += 1;
        }
    });

    thread::spawn(move || {
        match delete_all_documents(&liked_tracks_index) {
            Ok(_) => {}
            Err(e) => eprintln!("Error deleting all documents: {:?}", e),
        }
        let mut i = 1;
        let len = liked_tracks.len();
        for liked_track in liked_tracks {
            println!(
                "[{}/{}] Indexing liked track: {}",
                i,
                len,
                liked_track.title.bright_green()
            );
            index_entity::<LikedTrack>(&liked_tracks_index, &liked_track.into()).unwrap();
            i += 1;
        }
    });

    res.text("0");
    Ok(())
}
