use async_graphql::*;
use rockbox_library::{entity::favourites::Favourites, repo};
use sqlx::{Pool, Sqlite};

use crate::schema::objects::track::Track;

use super::objects::{album::Album, artist::Artist};

#[derive(Default)]
pub struct LibraryQuery;

#[Object]
impl LibraryQuery {
    async fn albums(&self, ctx: &Context<'_>) -> Result<Vec<Album>, Error> {
        let pool = ctx.data::<Pool<Sqlite>>()?;
        let results = repo::album::all(pool.clone()).await?;
        Ok(results.into_iter().map(Into::into).collect())
    }

    async fn artists(&self, ctx: &Context<'_>) -> Result<Vec<Artist>, Error> {
        let pool = ctx.data::<Pool<Sqlite>>()?;
        let results = repo::artist::all(pool.clone()).await?;
        Ok(results.into_iter().map(Into::into).collect())
    }

    async fn tracks(&self, ctx: &Context<'_>) -> Result<Vec<Track>, Error> {
        let pool = ctx.data::<Pool<Sqlite>>()?;
        let results = repo::track::all(pool.clone()).await?;
        Ok(results.into_iter().map(Into::into).collect())
    }

    async fn album(&self, ctx: &Context<'_>, id: String) -> Result<Option<Album>, Error> {
        let pool = ctx.data::<Pool<Sqlite>>()?;
        let results = repo::album::find(pool.clone(), &id).await?;
        let tracks = repo::album_tracks::find_by_album(pool.clone(), &id).await?;
        let mut album: Option<Album> = results.map(Into::into);
        if let Some(album) = album.as_mut() {
            album.tracks = tracks.into_iter().map(Into::into).collect();
        }
        Ok(album)
    }

    async fn artist(&self, ctx: &Context<'_>, id: String) -> Result<Option<Artist>, Error> {
        let pool = ctx.data::<Pool<Sqlite>>()?;
        let results = repo::artist::find(pool.clone(), &id).await?;
        let mut artist: Option<Artist> = results.map(Into::into);
        let albums = repo::album::find_by_artist(pool.clone(), &id).await?;
        let tracks = repo::artist_tracks::find_by_artist(pool.clone(), &id).await?;

        if let Some(artist) = artist.as_mut() {
            artist.albums = albums.into_iter().map(Into::into).collect();
            artist.tracks = tracks.into_iter().map(Into::into).collect();
        }

        Ok(artist)
    }

    async fn track(&self, ctx: &Context<'_>, id: String) -> Result<Option<Track>, Error> {
        let pool = ctx.data::<Pool<Sqlite>>()?;
        let results = repo::track::find(pool.clone(), &id).await?;
        Ok(results.map(Into::into))
    }

    async fn liked_tracks(&self, ctx: &Context<'_>) -> Result<Vec<Track>, Error> {
        let pool = ctx.data::<Pool<Sqlite>>()?;
        let results = repo::favourites::all_tracks(pool.clone()).await?;
        Ok(results.into_iter().map(Into::into).collect())
    }

    async fn liked_albums(&self, ctx: &Context<'_>) -> Result<Vec<Album>, Error> {
        let pool = ctx.data::<Pool<Sqlite>>()?;
        let results = repo::favourites::all_albums(pool.clone()).await?;
        Ok(results.into_iter().map(Into::into).collect())
    }
}

#[derive(Default)]
pub struct LibraryMutation;

#[Object]
impl LibraryMutation {
    async fn like_track(&self, ctx: &Context<'_>, id: String) -> Result<i32, Error> {
        let pool = ctx.data::<Pool<Sqlite>>()?;
        repo::favourites::save(
            pool.clone(),
            Favourites {
                id: cuid::cuid1()?,
                track_id: Some(id),
                created_at: chrono::Utc::now(),
                album_id: None,
            },
        )
        .await?;
        Ok(0)
    }

    async fn like_album(&self, ctx: &Context<'_>, id: String) -> Result<i32, Error> {
        let pool = ctx.data::<Pool<Sqlite>>()?;
        repo::favourites::save(
            pool.clone(),
            Favourites {
                id: cuid::cuid1()?,
                album_id: Some(id),
                created_at: chrono::Utc::now(),
                track_id: None,
            },
        )
        .await?;
        Ok(0)
    }

    async fn unlike_track(&self, ctx: &Context<'_>, id: String) -> Result<i32, Error> {
        let pool = ctx.data::<Pool<Sqlite>>()?;
        repo::favourites::delete(pool.clone(), &id).await?;
        Ok(0)
    }

    async fn unlike_album(&self, ctx: &Context<'_>, id: String) -> Result<i32, Error> {
        let pool = ctx.data::<Pool<Sqlite>>()?;
        repo::favourites::delete(pool.clone(), &id).await?;
        Ok(0)
    }
}
