use std::path::PathBuf;

use deno_core::{extension, op2};

extension!(
    rb_settings,
    ops = [op_get_global_settings],
    esm = ["src/settings/settings.js"],
);

pub fn get_declaration() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src/settings/lib.rb_settings.d.ts")
}

#[op2(async)]
pub async fn op_get_global_settings() {}
