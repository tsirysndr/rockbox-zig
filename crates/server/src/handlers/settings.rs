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
    web::block(move || {
        let _player_mutex = PLAYER_MUTEX.lock().unwrap();
        rb::with_kernel_lock(move || {
            if let Err(e) = rockbox_settings::load_settings(Some(settings)) {
                tracing::error!("update_global_settings: load_settings failed: {e}");
            } else if let Err(e) = rockbox_settings::write_settings() {
                tracing::error!("update_global_settings: write_settings failed: {e}");
            }
        });
    })
    .await
    .map_err(ErrorInternalServerError)?;
    Ok(HttpResponse::NoContent().finish())
}
