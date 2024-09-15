use std::path::PathBuf;

use deno_core::{extension, op2};

extension!(
    rb_browse,
    ops = [op_rockbox_browse],
    esm = ["src/browse/browse.js"],
);

pub fn get_declaration() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src/browse/lib.rb_browse.d.ts")
}

#[op2(async)]
pub async fn op_rockbox_browse() {
    println!("op_rockbox_browse ...");
}
