use async_graphql::*;

use crate::{rockbox_url, schema::objects::entry::Entry};

#[derive(Default)]
pub struct BrowseQuery;

#[Object]
impl BrowseQuery {
    async fn tree_get_entries(&self, ctx: &Context<'_>, path: String) -> Result<Vec<Entry>, Error> {
        let client = ctx.data::<reqwest::Client>().unwrap();
        let url = format!("{}/tree_entries?q={}", rockbox_url(), path);
        let response = client.get(&url).send().await?;
        let response = response.json::<Vec<Entry>>().await?;
        Ok(response)
    }
}
