use anyhow::Error;
use rockbox_sys::{self as rb, sound::pcm, types::user_settings::NewGlobalSettings};

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

    if let Some(music_dir) = settings.clone().music_dir {
        if let Ok(_) = std::fs::metadata(&music_dir) {
            std::env::set_var(
                "ROCKBOX_LIBRARY",
                music_dir.replace("$HOME", &std::env::var("HOME")?),
            );
        }
    }

    rb::settings::save_settings(settings.clone(), new_settings.is_none());

    if let Some(ref output) = settings.audio_output {
        if output == "fifo" {
            let path = settings.fifo_path.as_deref().unwrap_or("/tmp/rockbox.fifo");
            pcm::fifo_set_path(path);
            pcm::switch_sink(pcm::PCM_SINK_FIFO);
        } else if output == "airplay" {
            if let Some(ref host) = settings.airplay_host {
                let port = settings.airplay_port.unwrap_or(5000);
                pcm::airplay_set_host(host, port);
                pcm::switch_sink(pcm::PCM_SINK_AIRPLAY);
            }
        }
    }

    rb::settings::apply_audio_settings();

    let enabled = unsafe { rb::global_settings.eq_enabled };
    rb::sound::pcmbuf_set_low_latency(true);
    rb::sound::dsp::eq_enable(enabled);
    rb::sound::pcmbuf_set_low_latency(false);

    Ok(())
}

pub fn write_settings() -> Result<(), Error> {
    let settings = rb::settings::get_global_settings();
    let mut settings: NewGlobalSettings = settings.into();
    let home = std::env::var("HOME")?;

    settings.music_dir =
        Some(std::env::var("ROCKBOX_LIBRARY").unwrap_or(format!("{}/Music", home)));

    let content = toml::to_string(&settings)?;

    let path = format!("{}/.config/rockbox.org/settings.toml", home);
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
