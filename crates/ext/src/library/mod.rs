use deno_core::{error::AnyError, extension, op2};
use rockbox_library::{create_connection_pool, repo};

use crate::library::types::{Album, Artist, Track};

pub mod types;

extension!(
    rb_library,
    ops = [
        op_get_albums,
        op_get_artists,
        op_get_tracks,
        op_get_album,
        op_get_artist,
        op_get_track,
    ],
    esm = ["src/library/library.js"],
);

#[op2(async)]
#[serde]
pub async fn op_get_albums() -> Result<Vec<Album>, AnyError> {
    let pool = create_connection_pool().await?;
    let albums = repo::album::all(pool).await?;
    let albums = albums.into_iter().map(Into::into).collect();
    Ok(albums)
}

#[op2(async)]
#[serde]
pub async fn op_get_artists() -> Result<Vec<Artist>, AnyError> {
    let pool = create_connection_pool().await?;
    let artists = repo::artist::all(pool).await?;
    let artists = artists.into_iter().map(Into::into).collect();
    Ok(artists)
}

#[op2(async)]
#[serde]
pub async fn op_get_tracks() -> Result<Vec<Track>, AnyError> {
    let pool = create_connection_pool().await?;
    let tracks = repo::track::all(pool).await?;
    let tracks = tracks.into_iter().map(Into::into).collect();
    Ok(tracks)
}

#[op2(async)]
#[serde]
pub async fn op_get_album(#[string] id: String) -> Result<Option<Album>, AnyError> {
    let pool = create_connection_pool().await?;
    let album = repo::album::find(pool.clone(), &id).await?;
    let tracks = repo::album_tracks::find_by_album(pool, &id).await?;
    let mut album: Option<Album> = album.map(Into::into);
    if let Some(album) = album.as_mut() {
        album.tracks = tracks.into_iter().map(Into::into).collect();
    }
    Ok(album)
}

#[op2(async)]
#[serde]
pub async fn op_get_artist(#[string] id: String) -> Result<Option<Artist>, AnyError> {
    let pool = create_connection_pool().await?;
    let artist = repo::artist::find(pool.clone(), &id).await?;
    let mut artist: Option<Artist> = artist.map(Into::into);
    let albums = repo::album::find_by_artist(pool.clone(), &id).await?;
    let tracks = repo::artist_tracks::find_by_artist(pool, &id).await?;

    if let Some(artist) = artist.as_mut() {
        artist.albums = albums.into_iter().map(Into::into).collect();
        artist.tracks = tracks.into_iter().map(Into::into).collect();
    }

    Ok(artist)
}

#[op2(async)]
#[serde]
pub async fn op_get_track(#[string] id: String) -> Result<Option<Track>, AnyError> {
    let pool = create_connection_pool().await?;
    let track = repo::track::find(pool, &id).await?;
    Ok(track.map(Into::into))
}
