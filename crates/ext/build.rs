use deno_console::deno_console;
use deno_core::error::AnyError;
use deno_core::{extension, op2, OpState};
use std::env;
use std::path::PathBuf;

#[op2]
#[string]
fn op_script_version(
    _state: &mut OpState,
    #[string] _arg: &str,
) -> Result<Option<String>, AnyError> {
    Ok(Some("1".to_string()))
}

fn main() {
    extension!(
        rockbox,
        esm_entry_point = "ext:rockbox/src/bootstrap.js",
        esm = [
            "src/browse/browse.js",
            "src/console/console.js",
            "src/playback/playback.js",
            "src/playlist/playlist.js",
            "src/settings/settings.js",
            "src/sound/sound.js",
            "src/system/system.js",
            "src/bootstrap.js",
        ],
    );

    let out_dir = PathBuf::from(env::var_os("OUT_DIR").unwrap());
    let snapshot_path = out_dir.join("ROCKBOX_SNAPSHOT.bin");

    let snapshot = deno_core::snapshot::create_snapshot(
        deno_core::snapshot::CreateSnapshotOptions {
            cargo_manifest_dir: env!("CARGO_MANIFEST_DIR"),
            startup_snapshot: None,
            skip_op_registration: false,
            extensions: vec![
                rockbox::init_ops_and_esm(),
                deno_console::init_ops_and_esm(),
            ],
            with_runtime_cb: None,
            extension_transpiler: None,
        },
        None,
    )
    .unwrap();

    std::fs::write(snapshot_path, snapshot.output).unwrap();
}
