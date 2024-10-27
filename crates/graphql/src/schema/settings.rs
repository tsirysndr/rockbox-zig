use async_graphql::*;

use crate::{rockbox_url, schema::objects::user_settings::UserSettings};
use rockbox_sys as rb;

use super::objects::new_global_settings::NewGlobalSettings;

#[derive(Default)]
pub struct SettingsQuery;

#[Object]
impl SettingsQuery {
    async fn global_settings(&self, ctx: &Context<'_>) -> Result<UserSettings, Error> {
        let client = ctx.data::<reqwest::Client>().unwrap();
        let url = format!("{}/settings", rockbox_url());
        let settings = client
            .get(url)
            .send()
            .await?
            .json::<rb::types::user_settings::UserSettings>()
            .await?;
        Ok(settings.into())
    }
}

#[derive(Default)]
pub struct SettingsMutation;

#[Object]
impl SettingsMutation {
    async fn save_settings(
        &self,
        ctx: &Context<'_>,
        settings: NewGlobalSettings,
    ) -> Result<bool, Error> {
        let client = ctx.data::<reqwest::Client>().unwrap();
        let url = format!("{}/settings", rockbox_url());
        let response = client.put(url).json(&settings).send().await?;
        Ok(response.status().is_success())
    }
}
