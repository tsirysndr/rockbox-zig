use async_graphql::*;
use rockbox_sys::types::RockboxVersion;

use crate::rockbox_url;

use super::objects::system_status::SystemStatus;

#[derive(Default)]
pub struct SystemQuery;

#[Object]
impl SystemQuery {
    async fn rockbox_version(&self, ctx: &Context<'_>) -> Result<String, Error> {
        let client = ctx.data::<reqwest::Client>().unwrap();
        let url = format!("{}/version", rockbox_url());
        let version = client
            .get(url)
            .send()
            .await?
            .json::<RockboxVersion>()
            .await?
            .version;
        Ok(version)
    }

    async fn global_status(&self, ctx: &Context<'_>) -> Result<SystemStatus, Error> {
        let client = ctx.data::<reqwest::Client>().unwrap();
        let url = format!("{}/status", rockbox_url());
        let status = client.get(url).send().await?.json::<SystemStatus>().await?;
        Ok(status)
    }
}

#[derive(Default)]
pub struct SystemMutation;
