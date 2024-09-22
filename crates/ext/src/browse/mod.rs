use std::path::PathBuf;

use deno_core::{error::AnyError, extension, op2};
use rockbox_sys::types::tree::Entry;

use crate::rockbox_url;

extension!(
    rb_browse,
    ops = [op_rockbox_browse, op_tree_get_entries],
    esm = ["src/browse/browse.js"],
);

pub fn get_declaration() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src/browse/lib.rb_browse.d.ts")
}

#[op2(async)]
pub async fn op_rockbox_browse() {
    println!("op_rockbox_browse ...");
}

#[op2(async)]
#[serde]
pub async fn op_tree_get_entries(#[string] path: String) -> Result<Vec<Entry>, AnyError> {
    let client = reqwest::Client::new();
    let url = format!("{}/tree_entries?q={}", rockbox_url(), path);
    let response = client.get(&url).send().await?;
    let entries = response.json::<Vec<Entry>>().await?;
    Ok(entries)
}
