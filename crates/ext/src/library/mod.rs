use deno_core::{error::AnyError, extension, op2};
use rockbox_library::{
    create_connection_pool,
    entity::{album::Album, artist::Artist, track::Track},
    repo,
};

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
    Ok(albums)
}

#[op2(async)]
#[serde]
pub async fn op_get_artists() -> Result<Vec<Artist>, AnyError> {
    let pool = create_connection_pool().await?;
    let artists = repo::artist::all(pool).await?;
    Ok(artists)
}

#[op2(async)]
#[serde]
pub async fn op_get_tracks() -> Result<Vec<Track>, AnyError> {
    let pool = create_connection_pool().await?;
    let tracks = repo::track::all(pool).await?;
    Ok(tracks)
}

#[op2(async)]
#[serde]
pub async fn op_get_album(#[string] id: String) -> Result<Option<Album>, AnyError> {
    let pool = create_connection_pool().await?;
    let album = repo::album::find(pool, &id).await?;
    Ok(album)
}

#[op2(async)]
#[serde]
pub async fn op_get_artist(#[string] id: String) -> Result<Option<Artist>, AnyError> {
    let pool = create_connection_pool().await?;
    let artist = repo::artist::find(pool, &id).await?;
    Ok(artist)
}

#[op2(async)]
#[serde]
pub async fn op_get_track(#[string] id: String) -> Result<Option<Track>, AnyError> {
    let pool = create_connection_pool().await?;
    let track = repo::track::find(pool, &id).await?;
    Ok(track)
}
