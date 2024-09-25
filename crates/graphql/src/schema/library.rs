use async_graphql::*;
use rockbox_library::repo;
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
        Ok(results.map(Into::into))
    }

    async fn artist(&self, ctx: &Context<'_>, id: String) -> Result<Option<Artist>, Error> {
        let pool = ctx.data::<Pool<Sqlite>>()?;
        let results = repo::artist::find(pool.clone(), &id).await?;
        Ok(results.map(Into::into))
    }

    async fn track(&self, ctx: &Context<'_>, id: String) -> Result<Option<Track>, Error> {
        let pool = ctx.data::<Pool<Sqlite>>()?;
        let results = repo::track::find(pool.clone(), &id).await?;
        Ok(results.map(Into::into))
    }
}
