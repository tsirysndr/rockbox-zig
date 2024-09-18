use std::path::PathBuf;

use deno_core::{error::AnyError, extension, op2};
use rockbox_sys::types::{system_status::SystemStatus, RockboxVersion};

use crate::rockbox_url;

extension!(
    rb_system,
    ops = [op_get_global_status, op_get_rockbox_version,],
    esm = ["src/system/system.js"],
);

pub fn get_declaration() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src/system/lib.rb_system.d.ts")
}

#[op2(async)]
#[serde]
pub async fn op_get_global_status() -> Result<SystemStatus, AnyError> {
    let client = reqwest::Client::new();
    let url = format!("{}/status", rockbox_url());
    let response = client.get(&url).send().await?;
    let status = response.json::<SystemStatus>().await?;
    Ok(status)
}

#[op2(async)]
#[string]
pub async fn op_get_rockbox_version() -> Result<String, AnyError> {
    let client = reqwest::Client::new();
    let url = format!("{}/version", rockbox_url());
    let response = client.get(&url).send().await?;
    Ok(response.json::<RockboxVersion>().await?.version)
}
