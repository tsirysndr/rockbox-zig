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
        rockbox_settings::load_settings(Some(settings))?;
        rockbox_settings::write_settings()
    })
    .await
    .map_err(ErrorInternalServerError)?
    .map_err(ErrorInternalServerError)?;
    Ok(HttpResponse::NoContent().finish())
}
