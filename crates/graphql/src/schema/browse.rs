use async_graphql::*;

use crate::{rockbox_url, schema::objects::entry::Entry};

#[derive(Default)]
pub struct BrowseQuery;

#[Object]
impl BrowseQuery {
    async fn browse_id3(&self) -> String {
        "browse id3".to_string()
    }

    async fn tree_get_context(&self) -> String {
        "tree get context".to_string()
    }

    async fn tree_get_entries(&self, ctx: &Context<'_>, path: String) -> Result<Vec<Entry>, Error> {
        let client = ctx.data::<reqwest::Client>().unwrap();
        let url = format!("{}/tree_entries?q={}", rockbox_url(), path);
        let response = client.get(&url).send().await?;
        let response = response.json::<Vec<Entry>>().await?;
        Ok(response)
    }

    async fn tree_get_entry_at(&self, ctx: &Context<'_>) -> Result<Entry, Error> {
        todo!()
    }

    async fn rockbox_browse(&self) -> String {
        "rockbox browse".to_string()
    }
}
