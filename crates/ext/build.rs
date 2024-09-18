use ::deno_fetch::FetchPermissions;
use deno_console::deno_console;
use deno_core::error::AnyError;
use deno_core::{extension, op2, OpState};
use deno_fetch::deno_fetch;
use deno_net::NetPermissions;
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

#[derive(Clone)]
struct Permissions;

impl deno_websocket::WebSocketPermissions for Permissions {
    fn check_net_url(
        &mut self,
        _url: &deno_core::url::Url,
        _api_name: &str,
    ) -> Result<(), deno_core::error::AnyError> {
        unreachable!("snapshotting!")
    }
}

impl deno_web::TimersPermission for Permissions {
    fn allow_hrtime(&mut self) -> bool {
        unreachable!("snapshotting!")
    }
}

impl FetchPermissions for Permissions {
    fn check_net_url(
        &mut self,
        _url: &deno_core::url::Url,
        _api_name: &str,
    ) -> Result<(), deno_core::error::AnyError> {
        unreachable!("snapshotting!")
    }

    fn check_read(&mut self, _path: &std::path::Path, _api_name: &str) -> Result<(), AnyError> {
        unreachable!("snapshotting!")
    }
}

impl NetPermissions for Permissions {
    fn check_read(
        &mut self,
        _p: &std::path::Path,
        _api_name: &str,
    ) -> Result<(), deno_core::error::AnyError> {
        Ok(())
    }

    fn check_write(
        &mut self,
        _p: &std::path::Path,
        _api_name: &str,
    ) -> Result<(), deno_core::error::AnyError> {
        Ok(())
    }

    fn check_net<T: AsRef<str>>(
        &mut self,
        _host: &(T, Option<u16>),
        _api_name: &str,
    ) -> Result<(), deno_core::error::AnyError> {
        Ok(())
    }
}

fn main() {
    extension!(
        rockbox,
        esm_entry_point = "ext:rockbox/src/bootstrap.js",
        esm = [
            "src/browse/browse.js",
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
                deno_webidl::deno_webidl::init_ops_and_esm(),
                deno_console::init_ops_and_esm(),
                deno_url::deno_url::init_ops_and_esm(),
                deno_web::deno_web::init_ops_and_esm::<Permissions>(
                    Default::default(),
                    Default::default(),
                ),
                deno_fetch::init_ops_and_esm::<Permissions>(Default::default()),
                deno_net::deno_net::init_ops_and_esm::<Permissions>(None, None),
            ],
            with_runtime_cb: None,
            extension_transpiler: None,
        },
        None,
    )
    .unwrap();

    std::fs::write(snapshot_path, snapshot.output).unwrap();
}
