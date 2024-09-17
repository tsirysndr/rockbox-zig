use std::{env, rc::Rc};

use ::deno_fetch::FetchPermissions;
use browse::rb_browse;
use deno_ast::{MediaType, ParseParams};
use deno_core::{error::AnyError, ModuleLoadResponse, ModuleSourceCode};
use deno_net::NetPermissions;
use playback::rb_playback;
use playlist::rb_playlist;
use settings::rb_settings;
use sound::rb_sound;
use system::rb_system;

pub mod browse;
pub mod dir;
pub mod playback;
pub mod playlist;
pub mod settings;
pub mod sound;
pub mod system;
pub mod tagcache;

#[derive(Clone)]
struct Permissions;

impl deno_websocket::WebSocketPermissions for Permissions {
    fn check_net_url(
        &mut self,
        _url: &deno_core::url::Url,
        _api_name: &str,
    ) -> Result<(), deno_core::error::AnyError> {
        unreachable!("to be implemented!")
    }
}

impl deno_web::TimersPermission for Permissions {
    fn allow_hrtime(&mut self) -> bool {
        unreachable!("to be implemented!")
    }
}

impl FetchPermissions for Permissions {
    fn check_net_url(
        &mut self,
        _url: &deno_core::url::Url,
        _api_name: &str,
    ) -> Result<(), deno_core::error::AnyError> {
        unreachable!("to be implemented!")
    }

    fn check_read(&mut self, _path: &std::path::Path, _api_name: &str) -> Result<(), AnyError> {
        unreachable!("to be implemented!")
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

pub struct TsModuleLoader;

impl deno_core::ModuleLoader for TsModuleLoader {
    fn resolve(
        &self,
        specifier: &str,
        referrer: &str,
        _kind: deno_core::ResolutionKind,
    ) -> Result<deno_core::ModuleSpecifier, AnyError> {
        deno_core::resolve_import(specifier, referrer).map_err(|e| e.into())
    }

    fn load(
        &self,
        module_specifier: &deno_core::ModuleSpecifier,
        _maybe_referrer: Option<&reqwest::Url>,
        _is_dyn_import: bool,
        _requested_module_type: deno_core::RequestedModuleType,
    ) -> ModuleLoadResponse {
        let module_specifier = module_specifier.clone();

        let module_load = Box::pin(async move {
            let path = module_specifier.to_file_path().unwrap();

            let media_type = MediaType::from_path(&path);
            let (module_type, should_transpile) = match MediaType::from_path(&path) {
                MediaType::JavaScript | MediaType::Mjs | MediaType::Cjs => {
                    (deno_core::ModuleType::JavaScript, false)
                }
                MediaType::Jsx => (deno_core::ModuleType::JavaScript, true),
                MediaType::TypeScript
                | MediaType::Mts
                | MediaType::Cts
                | MediaType::Dts
                | MediaType::Dmts
                | MediaType::Dcts
                | MediaType::Tsx => (deno_core::ModuleType::JavaScript, true),
                MediaType::Json => (deno_core::ModuleType::Json, false),
                _ => panic!("Unknown extension {:?}", path.extension()),
            };

            let code = std::fs::read_to_string(&path)?;
            let code = if should_transpile {
                let parsed = deno_ast::parse_module(ParseParams {
                    specifier: module_specifier.clone(),
                    text: code.into(),
                    media_type,
                    capture_tokens: false,
                    scope_analysis: false,
                    maybe_syntax: None,
                })?;
                String::from_utf8(
                    parsed
                        .transpile(&Default::default(), &Default::default())?
                        .into_source()
                        .source,
                )?
            } else {
                code
            };
            let module = deno_core::ModuleSource::new(
                module_type,
                ModuleSourceCode::String(code.into()),
                &module_specifier,
                None,
            );
            Ok(module)
        });

        ModuleLoadResponse::Async(module_load)
    }
}

static RUNTIME_SNAPSHOT: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/ROCKBOX_SNAPSHOT.bin"));

pub async fn run_js(file_path: &str) -> Result<(), AnyError> {
    let main_module = deno_core::resolve_path(file_path, env::current_dir()?.as_path())?;
    let mut js_runtime = deno_core::JsRuntime::new(deno_core::RuntimeOptions {
        module_loader: Some(Rc::new(TsModuleLoader)),
        startup_snapshot: Some(RUNTIME_SNAPSHOT),
        extensions: vec![
            rb_browse::init_ops(),
            rb_playback::init_ops(),
            rb_playlist::init_ops(),
            rb_settings::init_ops(),
            rb_sound::init_ops(),
            rb_system::init_ops(),
            deno_webidl::deno_webidl::init_ops(),
            deno_console::deno_console::init_ops(),
            deno_url::deno_url::init_ops(),
            deno_web::deno_web::init_ops::<Permissions>(Default::default(), Default::default()),
            deno_fetch::deno_fetch::init_ops::<Permissions>(Default::default()),
            deno_net::deno_net::init_ops::<Permissions>(None, None),
        ],
        ..Default::default()
    });

    let mod_id = js_runtime.load_main_es_module(&main_module).await?;
    let result = js_runtime.mod_evaluate(mod_id);
    js_runtime.run_event_loop(Default::default()).await?;
    result.await
}

pub fn rockbox_url() -> String {
    let port = std::env::var("ROCKBOX_TCP_PORT").unwrap_or_else(|_| "6063".to_string());
    format!("http://127.0.0.1:{}", port)
}
