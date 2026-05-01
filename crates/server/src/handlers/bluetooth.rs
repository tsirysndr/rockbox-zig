use actix_web::{error::ErrorInternalServerError, web, HttpResponse};
use rockbox_bluetooth::{connect, disconnect, get_devices, scan};
use serde::Deserialize;

type HandlerResult = actix_web::Result<HttpResponse>;

#[derive(Deserialize)]
pub struct ScanQuery {
    timeout_secs: Option<u64>,
}

pub async fn scan_bluetooth(query: web::Query<ScanQuery>) -> HandlerResult {
    let timeout_secs = query.timeout_secs.unwrap_or(10);
    let devices = scan(timeout_secs).await.map_err(ErrorInternalServerError)?;
    Ok(HttpResponse::Ok().json(devices))
}

pub async fn get_bluetooth_devices() -> HandlerResult {
    let devices = get_devices().await.map_err(ErrorInternalServerError)?;
    Ok(HttpResponse::Ok().json(devices))
}

pub async fn connect_bluetooth_device(path: web::Path<String>) -> HandlerResult {
    let address = path.into_inner();
    match connect(&address).await {
        Ok(_) => Ok(HttpResponse::Ok().finish()),
        Err(e) => {
            tracing::error!("bluetooth: connect {}: {}", address, e);
            Ok(HttpResponse::InternalServerError().finish())
        }
    }
}

pub async fn disconnect_bluetooth_device(path: web::Path<String>) -> HandlerResult {
    let address = path.into_inner();
    match disconnect(&address).await {
        Ok(_) => Ok(HttpResponse::Ok().finish()),
        Err(e) => {
            tracing::error!("bluetooth: disconnect {}: {}", address, e);
            Ok(HttpResponse::InternalServerError().finish())
        }
    }
}
