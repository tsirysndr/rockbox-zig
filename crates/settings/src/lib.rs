use anyhow::Error;
use rockbox_sys::{
    self as rb,
    sound::{normalizer, pcm},
    types::user_settings::NewGlobalSettings,
};

pub fn load_settings(new_settings: Option<NewGlobalSettings>) -> Result<(), Error> {
    let settings: NewGlobalSettings = match new_settings.clone() {
        Some(settings) => settings,
        None => {
            let home = std::env::var("HOME")?;
            let path = format!("{}/.config/rockbox.org/settings.toml", home);
            let content = std::fs::read_to_string(&path)?;
            toml::from_str(&content)?
        }
    };

    if new_settings.is_none() {
        // disable sleep timer
        rb::system::set_sleeptimer_duration(0);
    }

    let home = std::env::var("HOME")?;
    let default_music_dir = format!("{}/Music", home);
    // Ensure the default music directory always exists.
    if let Err(e) = std::fs::create_dir_all(&default_music_dir) {
        tracing::warn!("could not create default music dir {default_music_dir}: {e}");
    }

    if let Some(music_dir) = settings.clone().music_dir {
        let resolved = music_dir.replace("$HOME", &home);
        if std::fs::metadata(&resolved).is_ok() {
            std::env::set_var("ROCKBOX_LIBRARY", &resolved);
        } else {
            // Configured dir doesn't exist yet; fall back to the default so
            // the library scanner has somewhere to point.
            std::env::set_var("ROCKBOX_LIBRARY", &default_music_dir);
        }
    } else {
        std::env::set_var("ROCKBOX_LIBRARY", &default_music_dir);
    }

    rb::settings::save_settings(settings.clone(), new_settings.is_none());

    match settings.audio_output.as_deref() {
        Some("fifo") => {
            let path = settings.fifo_path.as_deref().unwrap_or("/tmp/rockbox.fifo");
            pcm::fifo_set_path(path);
            pcm::switch_sink(pcm::PCM_SINK_FIFO);
            tracing::info!("audio output: fifo ({})", path);
        }
        Some("airplay") => {
            pcm::airplay_clear_receivers();
            // Multi-room list takes precedence over the legacy single-host fields.
            if let Some(ref receivers) = settings.airplay_receivers {
                if receivers.is_empty() {
                    tracing::warn!("audio output: airplay_receivers is empty");
                }
                for r in receivers {
                    let port = r.port.unwrap_or(5000);
                    pcm::airplay_add_receiver(&r.host, port);
                    tracing::info!("audio output: airplay receiver {}:{}", r.host, port);
                }
                pcm::switch_sink(pcm::PCM_SINK_AIRPLAY);
            } else if let Some(ref host) = settings.airplay_host {
                let port = settings.airplay_port.unwrap_or(5000);
                pcm::airplay_set_host(host, port);
                pcm::switch_sink(pcm::PCM_SINK_AIRPLAY);
                tracing::info!("audio output: airplay {}:{}", host, port);
            } else {
                tracing::warn!("audio output: airplay selected but no receiver configured");
            }
        }
        Some("squeezelite") => {
            let slim_port = settings.squeezelite_port.unwrap_or(3483);
            let http_port = settings.squeezelite_http_port.unwrap_or(9999);
            pcm::squeezelite_set_slim_port(slim_port);
            pcm::squeezelite_set_http_port(http_port);
            pcm::switch_sink(pcm::PCM_SINK_SQUEEZELITE);
            tracing::info!(
                "audio output: squeezelite (Slim Protocol :{slim_port}, HTTP audio :{http_port})"
            );
        }
        Some("upnp") => {
            let http_port = settings.upnp_http_port.unwrap_or(7879);
            pcm::upnp_set_http_port(http_port);
            if let Some(ref url) = settings.upnp_renderer_url {
                pcm::upnp_set_renderer_url(url);
                tracing::info!("audio output: upnp (WAV stream :{http_port}, renderer {url})");
            } else {
                pcm::upnp_clear_renderer_url();
                tracing::info!("audio output: upnp (WAV stream :{http_port})");
            }
            pcm::switch_sink(pcm::PCM_SINK_UPNP);
        }
        Some("chromecast") => {
            let http_port = settings.chromecast_http_port.unwrap_or(7881);
            pcm::chromecast_set_http_port(http_port);
            if let Some(ref host) = settings.chromecast_host {
                let device_port = settings.chromecast_port.unwrap_or(8009);
                pcm::chromecast_set_device_host(host);
                pcm::chromecast_set_device_port(device_port);
                tracing::info!(
                    "audio output: chromecast (WAV stream :{http_port}, device {}:{})",
                    host,
                    device_port
                );
            } else {
                tracing::warn!(
                    "audio output: chromecast selected but no chromecast_host configured"
                );
            }
            pcm::switch_sink(pcm::PCM_SINK_CHROMECAST);
        }
        Some("snapcast_tcp") => {
            if let Some(ref host) = settings.snapcast_tcp_host {
                let port = settings.snapcast_tcp_port.unwrap_or(4953);
                pcm::tcp_set_host(host);
                pcm::tcp_set_port(port);
                pcm::switch_sink(pcm::PCM_SINK_SNAPCAST_TCP);
                tracing::info!("audio output: snapcast_tcp ({}:{})", host, port);
            } else {
                tracing::warn!(
                    "audio output: snapcast_tcp selected but no snapcast_tcp_host configured"
                );
            }
        }
        Some("builtin") | None => {
            tracing::info!("audio output: builtin (SDL)");
        }
        Some(other) => {
            tracing::warn!(
                "audio output: unknown value {:?}, falling back to builtin",
                other
            );
        }
    }

    // Start UPnP/DLNA ContentDirectory media server if enabled.
    if settings.upnp_server_enabled.unwrap_or(false) {
        let port = settings.upnp_server_port.unwrap_or(7878);
        let name = settings.upnp_friendly_name.as_deref().unwrap_or("Rockbox");
        rockbox_upnp::start_media_server(port, name);
    }

    // Start UPnP/DLNA MediaRenderer:1 if enabled.
    if settings.upnp_renderer_enabled.unwrap_or(false) {
        let port = settings.upnp_renderer_port.unwrap_or(7880);
        let name = settings.upnp_friendly_name.as_deref().unwrap_or("Rockbox");
        rockbox_upnp::start_renderer(port, name);
    }

    normalizer::enable(settings.normalize_volume.unwrap_or(false));

    rb::settings::apply_audio_settings();

    let enabled = unsafe { rb::global_settings.eq_enabled };
    rb::sound::pcmbuf_set_low_latency(true);
    rb::sound::dsp::eq_enable(enabled);
    rb::sound::pcmbuf_set_low_latency(false);

    Ok(())
}

pub fn write_settings() -> Result<(), Error> {
    let from_c: NewGlobalSettings = rb::settings::get_global_settings().into();
    let home = std::env::var("HOME")?;

    // Start from whatever is already on disk so Rust-only fields
    // (audio_output, upnp_*, airplay_*, fifo_path, etc.) are never lost
    // when writing back only the C-firmware-owned settings.
    let mut settings = read_settings().unwrap_or_default();

    // Only update music_dir when ROCKBOX_LIBRARY was explicitly set at runtime.
    // If it was never set (e.g. the directory didn't exist at startup), keep
    // whatever the TOML already has to avoid resetting to the default ~/Music.
    if let Ok(library) = std::env::var("ROCKBOX_LIBRARY") {
        settings.music_dir = Some(library);
    }
    settings.playlist_shuffle = from_c.playlist_shuffle;
    settings.repeat_mode = from_c.repeat_mode;
    settings.bass = from_c.bass;
    settings.treble = from_c.treble;
    settings.bass_cutoff = from_c.bass_cutoff;
    settings.treble_cutoff = from_c.treble_cutoff;
    settings.crossfade = from_c.crossfade;
    settings.fade_on_stop = from_c.fade_on_stop;
    settings.fade_in_delay = from_c.fade_in_delay;
    settings.fade_in_duration = from_c.fade_in_duration;
    settings.fade_out_delay = from_c.fade_out_delay;
    settings.fade_out_duration = from_c.fade_out_duration;
    settings.fade_out_mixmode = from_c.fade_out_mixmode;
    settings.balance = from_c.balance;
    settings.stereo_width = from_c.stereo_width;
    settings.stereosw_mode = from_c.stereosw_mode;
    settings.surround_enabled = from_c.surround_enabled;
    settings.surround_balance = from_c.surround_balance;
    settings.surround_fx1 = from_c.surround_fx1;
    settings.surround_fx2 = from_c.surround_fx2;
    settings.party_mode = from_c.party_mode;
    settings.channel_config = from_c.channel_config;
    settings.player_name = from_c.player_name;
    settings.eq_enabled = from_c.eq_enabled;
    settings.eq_band_settings = from_c.eq_band_settings;
    settings.replaygain_settings = from_c.replaygain_settings;
    settings.compressor_settings = from_c.compressor_settings;
    settings.normalize_volume = Some(normalizer::is_enabled());

    let content = toml::to_string(&settings)?;
    let path = format!("{}/.config/rockbox.org/settings.toml", home);
    std::fs::write(&path, content)?;
    Ok(())
}

/// Read the current settings.toml without applying anything.
pub fn read_settings() -> Result<NewGlobalSettings, Error> {
    let home = std::env::var("HOME")?;
    let path = format!("{}/.config/rockbox.org/settings.toml", home);
    match std::fs::read_to_string(&path) {
        Ok(content) => Ok(toml::from_str(&content)?),
        Err(_) => Ok(NewGlobalSettings::default()),
    }
}

/// Persist `settings` to settings.toml as-is (all fields preserved, no C
/// firmware involvement).  Use this instead of `write_settings()` when you
/// want to save audio-output-related fields that the C firmware doesn't know
/// about.
pub fn save_settings_to_file(settings: &NewGlobalSettings) -> Result<(), Error> {
    let home = std::env::var("HOME")?;
    let path = format!("{}/.config/rockbox.org/settings.toml", home);
    let content = toml::to_string(settings)?;
    std::fs::write(&path, content)?;
    Ok(())
}

pub fn get_music_dir() -> Result<String, Error> {
    let home = std::env::var("HOME")?;
    let path = format!("{}/.config/rockbox.org/settings.toml", home);

    if let Err(_) = std::fs::metadata(&path) {
        return Ok(std::env::var("ROCKBOX_LIBRARY").unwrap_or(format!("{}/Music", home)));
    }

    let content = std::fs::read_to_string(&path)?;
    let settings: NewGlobalSettings = toml::from_str(&content)?;
    let music_dir = std::env::var("ROCKBOX_LIBRARY").unwrap_or(format!("{}/Music", home));
    Ok(settings.music_dir.unwrap_or(music_dir))
}

#[cfg(test)]
mod tests {
    use rockbox_sys::types::user_settings::{CompressorSettings, NewGlobalSettings};

    #[test]
    fn compressor_settings_round_trip() {
        let original = NewGlobalSettings {
            compressor_settings: Some(CompressorSettings {
                threshold: -24,
                makeup_gain: 1,
                ratio: 4,
                knee: 1,
                release_time: 300,
                attack_time: 5,
            }),
            ..Default::default()
        };

        let toml_str = toml::to_string(&original).expect("serialize");
        assert!(toml_str.contains("[compressor_settings]"));

        let restored: NewGlobalSettings = toml::from_str(&toml_str).expect("deserialize");
        let c = restored
            .compressor_settings
            .expect("compressor_settings present");
        assert_eq!(c.threshold, -24);
        assert_eq!(c.makeup_gain, 1);
        assert_eq!(c.ratio, 4);
        assert_eq!(c.knee, 1);
        assert_eq!(c.release_time, 300);
        assert_eq!(c.attack_time, 5);
    }

    #[test]
    fn compressor_settings_absent_when_none() {
        let settings = NewGlobalSettings {
            compressor_settings: None,
            ..Default::default()
        };
        let toml_str = toml::to_string(&settings).expect("serialize");
        assert!(!toml_str.contains("compressor_settings"));

        let restored: NewGlobalSettings = toml::from_str(&toml_str).expect("deserialize");
        assert!(restored.compressor_settings.is_none());
    }

    #[test]
    fn existing_toml_without_compressor_deserializes_to_none() {
        let toml_str = r#"
music_dir = "/home/user/Music"
playlist_shuffle = false
repeat_mode = 1

[replaygain_settings]
noclip = false
type = 0
preamp = -15
"#;
        let settings: NewGlobalSettings = toml::from_str(toml_str).expect("deserialize");
        assert!(settings.compressor_settings.is_none());
        assert_eq!(settings.repeat_mode, Some(1));
    }
}
