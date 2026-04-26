use anyhow::Error;
use rockbox_chromecast::Chromecast;
use rockbox_settings::{read_settings, save_settings_to_file};
use rockbox_sys::sound::pcm;
use std::thread;

use crate::{
    http::{Context, Request, Response},
    GLOBAL_MUTEX,
};

pub async fn connect(ctx: &Context, req: &Request, res: &mut Response) -> Result<(), Error> {
    let id = &req.params[0];
    let mut player = ctx.player.lock().unwrap();
    let mut current_device = ctx.current_device.lock().unwrap();
    let mut devices = ctx.devices.lock().unwrap();

    let device = match devices.iter().find(|d| d.id == *id).cloned().or_else(|| {
        // Synthetic device from settings (mDNS not yet found it).
        current_device.as_ref().filter(|d| d.id == *id).cloned()
    }) {
        Some(d) => d,
        None => {
            res.set_status(404);
            return Ok(());
        }
    };

    // Stop any existing player session (e.g. Chromecast).
    if let Some(p) = player.as_mut() {
        let _ = p.stop().await;
        let _ = p.disconnect().await;
    }
    *player = None;

    // Read current settings so we preserve all other fields.
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
            *GLOBAL_MUTEX.lock().unwrap() = 1;
        }
        other => {
            tracing::warn!("connect: unknown device service {:?}", other);
            res.set_status(400);
            return Ok(());
        }
    }

    // Persist and update state before any potentially-failing connection attempt
    // so that the selection survives even if e.g. the Chromecast is temporarily unreachable.
    if let Err(e) = save_settings_to_file(&settings) {
        tracing::warn!("connect: failed to save settings: {e}");
    }

    // Mark new current device; clear is_current_device on all others.
    for d in devices.iter_mut() {
        d.is_current_device = d.id == device.id;
    }
    *current_device = Some(device.clone());

    // For Chromecast, establish the player session in a background thread so this
    // handler returns immediately (the TCP + RTSP handshake can take several seconds).
    // The PCM sink is already armed; settings are already saved.
    if device.service == "chromecast" {
        let player_arc = ctx.player.clone();
        thread::spawn(move || match Chromecast::connect(device) {
            Ok(p) => *player_arc.lock().unwrap() = p,
            Err(e) => tracing::warn!("chromecast: connect failed (sink still armed): {e}"),
        });
    }

    res.set_status(200);
    Ok(())
}

pub async fn disconnect(ctx: &Context, req: &Request, res: &mut Response) -> Result<(), Error> {
    let _id = &req.params[0];
    let mut player = ctx.player.lock().unwrap();
    let mut current_device = ctx.current_device.lock().unwrap();
    let mut devices = ctx.devices.lock().unwrap();

    if let Some(p) = player.as_mut() {
        let _ = p.stop().await;
        let _ = p.disconnect().await;
    }
    *GLOBAL_MUTEX.lock().unwrap() = 0;
    *player = None;

    // Fall back to built-in sink.
    pcm::switch_sink(pcm::PCM_SINK_BUILTIN);

    let mut settings = read_settings().unwrap_or_default();
    settings.audio_output = Some("builtin".to_string());
    if let Err(e) = save_settings_to_file(&settings) {
        tracing::warn!("disconnect: failed to save settings: {e}");
    }

    // Mark built-in as current.
    for d in devices.iter_mut() {
        d.is_current_device = d.id == "builtin";
    }
    *current_device = devices.iter().find(|d| d.id == "builtin").cloned();

    res.set_status(200);
    Ok(())
}

pub async fn get_devices(ctx: &Context, _req: &Request, res: &mut Response) -> Result<(), Error> {
    let current = ctx.current_device.lock().unwrap().clone();
    let devices = ctx.devices.lock().unwrap();

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

    // If the current device isn't in the discovered list yet (e.g. Chromecast
    // from settings but mDNS hasn't found it), include it so UIs can show it.
    if let Some(ref cd) = current {
        if !result.iter().any(|d| devices_match(cd, d)) {
            result.push(cd.clone());
        }
    }

    res.json(&result);
    Ok(())
}

/// Two devices represent the same physical output if their IDs match OR if
/// their service + IP match (handles ID format differences between settings-
/// based synthetic devices and mDNS-discovered ones).
fn devices_match(a: &rockbox_types::device::Device, b: &rockbox_types::device::Device) -> bool {
    if a.id == b.id {
        return true;
    }
    if a.service != b.service {
        return false;
    }
    match a.service.as_str() {
        "builtin" | "fifo" | "squeezelite" => true,
        _ => !a.ip.is_empty() && a.ip == b.ip,
    }
}

pub async fn get_device(ctx: &Context, req: &Request, res: &mut Response) -> Result<(), Error> {
    let id = &req.params[0];
    if id == "current" {
        let current_device = ctx.current_device.lock().unwrap();
        if let Some(device) = current_device.as_ref() {
            res.json(&device.clone());
            return Ok(());
        }
        res.set_status(404);
        return Ok(());
    }

    let devices = ctx.devices.lock().unwrap();
    let device = devices.iter().find(|d| d.id == *id);

    if let Some(device) = device {
        res.json(&device.clone());
        return Ok(());
    }

    res.json(&device);
    res.set_status(404);
    Ok(())
}
