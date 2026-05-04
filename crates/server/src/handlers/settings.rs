use actix_web::{error::ErrorInternalServerError, web, HttpResponse};
use rockbox_sys as rb;
use rockbox_sys::types::user_settings::NewGlobalSettings;

use crate::PLAYER_MUTEX;

type HandlerResult = actix_web::Result<HttpResponse>;

pub async fn get_global_settings() -> HandlerResult {
    let settings = web::block(|| {
        let _player_mutex = PLAYER_MUTEX.lock().unwrap();
        rb::settings::get_global_settings()
    })
    .await
    .map_err(ErrorInternalServerError)?;
    Ok(HttpResponse::Ok().json(settings))
}

pub async fn update_global_settings(body: web::Json<NewGlobalSettings>) -> HandlerResult {
    let settings = body.into_inner();
    // Route the settings apply through the firmware-command bus — most
    // settings just write a value, but `crossfade` calls
    // `audio_set_crossfade()` which queues `Q_AUDIO_REMAKE_AUDIO_BUFFER`
    // and ends up in the kernel scheduler. From an actix worker the
    // scheduler reads the wrong `__cores[0].running` slot and corrupts
    // kernel-thread state — see crates/server/src/fw_bus.rs. Run the
    // whole load_settings on the broker (a real Rockbox kernel thread)
    // so the FFI calls resolve to a sane current-thread.
    web::block(move || {
        let _player_mutex = PLAYER_MUTEX.lock().unwrap();
        crate::fw_bus::send_and_wait(|reply| {
            crate::fw_bus::FwCmd::Custom(Box::new(move || {
                if let Err(e) = rockbox_settings::load_settings(Some(settings)) {
                    tracing::error!("update_global_settings: load_settings failed: {e}");
                } else if let Err(e) = rockbox_settings::write_settings() {
                    tracing::error!("update_global_settings: write_settings failed: {e}");
                }
                let _ = reply.send(());
            }))
        });
    })
    .await
    .map_err(ErrorInternalServerError)?;
    Ok(HttpResponse::NoContent().finish())
}
