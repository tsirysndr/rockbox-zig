use anyhow::Error;
use rockbox_sys::{self as rb, types::user_settings::NewGlobalSettings};

pub fn load_settings(settings: Option<NewGlobalSettings>) -> Result<(), Error> {
    let settings: NewGlobalSettings = match settings {
        Some(settings) => settings,
        None => {
            let home = std::env::var("HOME")?;
            let path = format!("{}/.config/rockbox.org/settings.toml", home);
            let content = std::fs::read_to_string(&path)?;
            toml::from_str(&content)?
        }
    };

    if let Some(music_dir) = settings.clone().music_dir {
        if let Ok(_) = std::fs::metadata(&music_dir) {
            std::env::set_var(
                "ROCKBOX_LIBRARY",
                music_dir.replace("$HOME", &std::env::var("HOME")?),
            );
        }
    }

    rb::settings::save_settings(settings.clone());

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
