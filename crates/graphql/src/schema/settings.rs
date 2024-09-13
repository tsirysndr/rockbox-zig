use async_graphql::*;

use crate::{rockbox_url, schema::objects::user_settings::UserSettings};

#[derive(Default)]
pub struct SettingsQuery;

#[Object]
impl SettingsQuery {
    async fn global_settings(&self, ctx: &Context<'_>) -> Result<UserSettings, Error> {
        let client = ctx.data::<reqwest::Client>().unwrap();
        let url = format!("{}/settings", rockbox_url());
        let settings = client.get(url).send().await?.json::<UserSettings>().await?;
        Ok(settings)
    }
}

#[derive(Default)]
pub struct SettingsMutation;
