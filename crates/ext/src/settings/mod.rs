use std::path::PathBuf;

use deno_core::{error::AnyError, extension, op2};
use rockbox_sys::types::user_settings::UserSettings;

use crate::rockbox_url;

extension!(
    rb_settings,
    ops = [op_get_global_settings],
    esm = ["src/settings/settings.js"],
);

pub fn get_declaration() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src/settings/lib.rb_settings.d.ts")
}

#[op2(async)]
#[serde]
pub async fn op_get_global_settings() -> Result<UserSettings, AnyError> {
    let client = reqwest::Client::new();
    let url = format!("{}/settings", rockbox_url());
    let response = client.get(&url).send().await?;
    let settings = response.json::<UserSettings>().await?;
    Ok(settings)
}
