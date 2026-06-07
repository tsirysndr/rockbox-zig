use actix_web::{web, HttpResponse};
use rockbox_settings::{read_settings, save_settings_to_file};
use rockbox_sys::sound::pcm;

use crate::{http::AppState, GLOBAL_MUTEX};

type HandlerResult = actix_web::Result<HttpResponse>;

pub async fn connect(state: web::Data<AppState>, path: web::Path<String>) -> HandlerResult {
    let id = path.into_inner();
    let mut player = state.player.lock().unwrap();
    let mut current_device = state.current_device.lock().unwrap();
    let mut devices = state.devices.lock().unwrap();

    let device = match devices
        .iter()
        .find(|d| d.id == id)
        .cloned()
        .or_else(|| current_device.as_ref().filter(|d| d.id == id).cloned())
    {
        Some(d) => d,
        None => return Ok(HttpResponse::NotFound().finish()),
    };

    if let Some(p) = player.as_mut() {
        let _ = p.stop().await;
        let _ = p.disconnect().await;
    }
    *player = None;

    let old_service = current_device
        .as_ref()
        .map(|d| d.service.as_str())
        .unwrap_or("");
    if old_service == "chromecast" || device.service == "chromecast" {
        pcm::chromecast_teardown();
    }

    let mut settings = read_settings().unwrap_or_default();

    match device.service.as_str() {
        "builtin" => {
            settings.audio_output = Some("builtin".to_string());
            pcm::switch_sink(pcm::PCM_SINK_BUILTIN);
            *GLOBAL_MUTEX.lock().unwrap() = 0;
        }
        "fifo" => {
            settings.audio_output = Some("fifo".to_string());
            let path = settings
                .fifo_path
                .as_deref()
                .unwrap_or("/tmp/rockbox.fifo")
                .to_string();
            pcm::fifo_set_path(&path);
            pcm::switch_sink(pcm::PCM_SINK_FIFO);
            *GLOBAL_MUTEX.lock().unwrap() = 0;
        }
        "airplay" => {
            settings.audio_output = Some("airplay".to_string());
            settings.airplay_host = Some(device.ip.clone());
            settings.airplay_port = Some(device.port);
            pcm::airplay_clear_receivers();
            pcm::airplay_set_host(&device.ip, device.port);
            pcm::switch_sink(pcm::PCM_SINK_AIRPLAY);
            *GLOBAL_MUTEX.lock().unwrap() = 0;
        }
        "squeezelite" => {
            let slim_port = settings.squeezelite_port.unwrap_or(3483);
            let http_port = settings.squeezelite_http_port.unwrap_or(9999);
            settings.audio_output = Some("squeezelite".to_string());
            settings.squeezelite_port = Some(slim_port);
            settings.squeezelite_http_port = Some(http_port);
            pcm::squeezelite_set_slim_port(slim_port);
            pcm::squeezelite_set_http_port(http_port);
            pcm::switch_sink(pcm::PCM_SINK_SQUEEZELITE);
            *GLOBAL_MUTEX.lock().unwrap() = 0;
        }
        "upnp" => {
            let http_port = settings.upnp_http_port.unwrap_or(7879);
            settings.audio_output = Some("upnp".to_string());
            if let Some(ref url) = device.base_url {
                settings.upnp_renderer_url = Some(url.clone());
                pcm::upnp_set_renderer_url(url);
            }
            pcm::upnp_set_http_port(http_port);
            pcm::upnp_reset_renderer();
            pcm::switch_sink(pcm::PCM_SINK_UPNP);
            *GLOBAL_MUTEX.lock().unwrap() = 0;
        }
        "chromecast" => {
            let http_port = settings.chromecast_http_port.unwrap_or(7881);
            settings.audio_output = Some("chromecast".to_string());
            settings.chromecast_host = Some(device.ip.clone());
            settings.chromecast_port = Some(device.port);
            pcm::chromecast_set_http_port(http_port);
            pcm::chromecast_set_device_host(&device.ip);
            pcm::chromecast_set_device_port(device.port);
            pcm::switch_sink(pcm::PCM_SINK_CHROMECAST);
            *GLOBAL_MUTEX.lock().unwrap() = 0;
        }
        "snapcast" => {
            settings.audio_output = Some("snapcast_tcp".to_string());
            settings.snapcast_tcp_host = Some(device.ip.clone());
            settings.snapcast_tcp_port = Some(device.port);
            pcm::tcp_set_host(&device.ip);
            pcm::tcp_set_port(device.port);
            pcm::switch_sink(pcm::PCM_SINK_SNAPCAST_TCP);
            *GLOBAL_MUTEX.lock().unwrap() = 0;
        }
        "cmaf" => {
            let http_port = settings.cmaf_http_port.unwrap_or(7882);
            let bitrate = settings.cmaf_bitrate.unwrap_or(128_000);
            settings.audio_output = Some("cmaf".to_string());
            settings.cmaf_http_port = Some(http_port);
            settings.cmaf_bitrate = Some(bitrate);
            pcm::cmaf_set_http_port(http_port);
            pcm::cmaf_set_bitrate(bitrate);
            pcm::cmaf_set_segment_dir(settings.cmaf_segment_dir.as_deref());
            pcm::switch_sink(pcm::PCM_SINK_CMAF);
            *GLOBAL_MUTEX.lock().unwrap() = 0;
        }
        other => {
            tracing::warn!("connect: unknown device service {:?}", other);
            return Ok(HttpResponse::BadRequest().finish());
        }
    }

    if let Err(e) = save_settings_to_file(&settings) {
        tracing::warn!("connect: failed to save settings: {e}");
    }

    for d in devices.iter_mut() {
        d.is_current_device = d.id == device.id;
    }
    *current_device = Some(device);

    Ok(HttpResponse::Ok().finish())
}

pub async fn disconnect(state: web::Data<AppState>, _path: web::Path<String>) -> HandlerResult {
    let mut player = state.player.lock().unwrap();
    let mut current_device = state.current_device.lock().unwrap();
    let mut devices = state.devices.lock().unwrap();

    if let Some(p) = player.as_mut() {
        let _ = p.stop().await;
        let _ = p.disconnect().await;
    }
    *GLOBAL_MUTEX.lock().unwrap() = 0;
    *player = None;

    if current_device
        .as_ref()
        .map_or(false, |d| d.service == "chromecast")
    {
        pcm::chromecast_teardown();
    }

    pcm::switch_sink(pcm::PCM_SINK_BUILTIN);

    let mut settings = read_settings().unwrap_or_default();
    settings.audio_output = Some("builtin".to_string());
    if let Err(e) = save_settings_to_file(&settings) {
        tracing::warn!("disconnect: failed to save settings: {e}");
    }

    for d in devices.iter_mut() {
        d.is_current_device = d.id == "builtin";
    }
    *current_device = devices.iter().find(|d| d.id == "builtin").cloned();

    Ok(HttpResponse::Ok().finish())
}

pub async fn get_devices(state: web::Data<AppState>) -> HandlerResult {
    let current = state.current_device.lock().unwrap().clone();
    let devices = state.devices.lock().unwrap();

    let mut result: Vec<_> = devices
        .iter()
        .map(|d| {
            let mut d = d.clone();
            d.is_current_device = current
                .as_ref()
                .map(|c| devices_match(c, &d))
                .unwrap_or(false);
            d
        })
        .collect();

    if let Some(ref cd) = current {
        if !result.iter().any(|d| devices_match(cd, d)) {
            result.push(cd.clone());
        }
    }

    Ok(HttpResponse::Ok().json(result))
}

fn devices_match(a: &rockbox_types::device::Device, b: &rockbox_types::device::Device) -> bool {
    if a.id == b.id {
        return true;
    }
    if a.service != b.service {
        return false;
    }
    match a.service.as_str() {
        "builtin" | "fifo" | "squeezelite" | "cmaf" => true,
        _ => !a.ip.is_empty() && a.ip == b.ip,
    }
}

pub async fn get_device(state: web::Data<AppState>, path: web::Path<String>) -> HandlerResult {
    let id = path.into_inner();
    if id == "current" {
        let current_device = state.current_device.lock().unwrap();
        return match current_device.as_ref() {
            Some(device) => Ok(HttpResponse::Ok().json(device)),
            None => Ok(HttpResponse::NotFound().finish()),
        };
    }

    let devices = state.devices.lock().unwrap();
    match devices.iter().find(|d| d.id == id) {
        Some(device) => Ok(HttpResponse::Ok().json(device)),
        None => Ok(HttpResponse::NotFound().finish()),
    }
}
