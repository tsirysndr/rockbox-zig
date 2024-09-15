use std::path::PathBuf;

use deno_core::{extension, op2};

extension!(
    rb_system,
    ops = [op_get_global_status, op_get_rockbox_version,],
    esm = ["src/system/system.js"],
);

pub fn get_declaration() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src/system/lib.rb_system.d.ts")
}

#[op2(async)]
pub async fn op_get_global_status() {}

#[op2(async)]
pub async fn op_get_rockbox_version() {}
