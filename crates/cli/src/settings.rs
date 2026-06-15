use anyhow::{anyhow, Error};
use serde::{Deserialize, Serialize};

const XRPC_BASE: &str = "https://api.rocksky.app/xrpc";

// ── Rocksky API types ─────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SettingsView {
    crossfade: Option<CrossfadeSettings>,
    equalizer: Option<EqualizerSettings>,
    replay_gain: Option<ReplayGainSettings>,
    tone: Option<ToneSettings>,
}

#[derive(Debug, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
struct CrossfadeSettings {
    #[serde(skip_serializing_if = "Option::is_none")]
    mode: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    fade_in_delay: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    fade_in_duration: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    fade_out_delay: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    fade_out_duration: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    fade_out_mix_mode: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct EqualizerBand {
    #[serde(skip_serializing_if = "Option::is_none")]
    frequency: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    gain: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    q: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize, Default)]
struct EqualizerSettings {
    #[serde(skip_serializing_if = "Option::is_none")]
    enabled: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    precut: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    bands: Option<Vec<EqualizerBand>>,
}

#[derive(Debug, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
struct ReplayGainSettings {
    #[serde(skip_serializing_if = "Option::is_none")]
    mode: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    preamp: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    prevent_clipping: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, Default)]
struct ToneSettings {
    #[serde(skip_serializing_if = "Option::is_none")]
    bass: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    treble: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    balance: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    channels: Option<String>,
}

// ── Path helpers ──────────────────────────────────────────────────────────────

fn config_dir() -> std::path::PathBuf {
    dirs::home_dir()
        .expect("home directory not found")
        .join(".config/rockbox.org")
}

pub fn load_token() -> Result<String, Error> {
    std::fs::read_to_string(config_dir().join("token"))
        .map(|s| s.trim().to_string())
        .map_err(|_| anyhow!("Not logged in. Use `rockboxd login <handle>` first."))
}

fn settings_path() -> std::path::PathBuf {
    config_dir().join("settings.toml")
}

// ── Enum mappings ─────────────────────────────────────────────────────────────

fn crossfade_mode_from_int(v: i64) -> &'static str {
    match v {
        0 => "disabled",
        1 => "enabled",
        2 => "shuffle",
        3 => "albumChange",
        4 => "trackChange",
        _ => "disabled",
    }
}

fn crossfade_mode_to_int(s: &str) -> i64 {
    match s {
        "disabled" => 0,
        "enabled" => 1,
        "shuffle" => 2,
        "albumChange" => 3,
        "trackChange" => 4,
        _ => 0,
    }
}

fn mix_mode_from_int(v: i64) -> &'static str {
    match v {
        1 => "mix",
        _ => "crossfade",
    }
}

fn mix_mode_to_int(s: &str) -> i64 {
    match s {
        "mix" => 1,
        _ => 0,
    }
}

fn replay_gain_mode_from_int(v: i64) -> &'static str {
    match v {
        0 => "track",
        1 => "album",
        3 => "trackIfShuffling",
        _ => "disabled",
    }
}

fn replay_gain_mode_to_int(s: &str) -> i64 {
    match s {
        "track" => 0,
        "album" => 1,
        "trackIfShuffling" => 3,
        _ => 2,
    }
}

fn channel_config_from_int(v: i64) -> &'static str {
    match v {
        1 => "mono",
        2 => "monoLeft",
        3 => "monoRight",
        4 => "karaoke",
        5 => "wide",
        _ => "stereo",
    }
}

fn channel_config_to_int(s: &str) -> i64 {
    match s {
        "mono" => 1,
        "monoLeft" => 2,
        "monoRight" => 3,
        "karaoke" => 4,
        "wide" => 5,
        _ => 0,
    }
}

// ── Auth guard ───────────────────────────────────────────────────────────────

fn check_auth(res: &reqwest::Response) -> Result<(), Error> {
    if res.status() == reqwest::StatusCode::UNAUTHORIZED
        || res.status() == reqwest::StatusCode::FORBIDDEN
    {
        Err(anyhow!(
            "Session expired or invalid. Run `rockboxd login <handle>` again."
        ))
    } else {
        Ok(())
    }
}

// ── pull ──────────────────────────────────────────────────────────────────────

/// Pull audio settings from Rocksky.
///
/// If `did` is `Some` the request is public — no token required.
/// If `did` is `None` a stored token is required (own settings).
pub async fn pull(did: Option<String>) -> Result<(), Error> {
    let mut url = format!("{}/app.rocksky.rockbox.getAudioSettings", XRPC_BASE);
    if let Some(ref d) = did {
        url = format!("{}?did={}", url, d);
    }

    let mut req = reqwest::Client::new().get(&url);

    if did.is_none() {
        let token = load_token()?;
        req = req.header("Authorization", format!("Bearer {}", token));
    }

    let res = req.send().await?;

    check_auth(&res)?;
    if !res.status().is_success() {
        return Err(anyhow!("API error {}: {}", res.status(), res.text().await?));
    }

    let view: SettingsView = res.json().await?;
    let path = settings_path();

    // Preserve all other fields in settings.toml; only update audio sections.
    let mut table: toml::map::Map<String, toml::Value> = if path.exists() {
        toml::from_str(&std::fs::read_to_string(&path)?).unwrap_or_default()
    } else {
        std::fs::create_dir_all(config_dir())?;
        toml::map::Map::new()
    };

    if let Some(cf) = &view.crossfade {
        if let Some(m) = &cf.mode {
            table.insert(
                "crossfade".into(),
                toml::Value::Integer(crossfade_mode_to_int(m)),
            );
        }
        macro_rules! set_int {
            ($key:expr, $val:expr) => {
                if let Some(v) = $val {
                    table.insert($key.into(), toml::Value::Integer(*v));
                }
            };
        }
        set_int!("fade_in_delay", cf.fade_in_delay.as_ref());
        set_int!("fade_in_duration", cf.fade_in_duration.as_ref());
        set_int!("fade_out_delay", cf.fade_out_delay.as_ref());
        set_int!("fade_out_duration", cf.fade_out_duration.as_ref());
        if let Some(m) = &cf.fade_out_mix_mode {
            table.insert(
                "fade_out_mixmode".into(),
                toml::Value::Integer(mix_mode_to_int(m)),
            );
        }
    }

    if let Some(eq) = &view.equalizer {
        if let Some(v) = eq.enabled {
            table.insert("eq_enabled".into(), toml::Value::Boolean(v));
        }
        if let Some(v) = eq.precut {
            table.insert("eq_precut".into(), toml::Value::Integer(v));
        }
        if let Some(bands) = &eq.bands {
            let toml_bands: Vec<toml::Value> = bands
                .iter()
                .map(|b| {
                    let mut m = toml::map::Map::new();
                    // API field `frequency` maps to TOML field `cutoff`
                    if let Some(f) = b.frequency {
                        m.insert("cutoff".into(), toml::Value::Integer(f));
                    }
                    if let Some(g) = b.gain {
                        m.insert("gain".into(), toml::Value::Integer(g));
                    }
                    if let Some(q) = b.q {
                        m.insert("q".into(), toml::Value::Integer(q));
                    }
                    toml::Value::Table(m)
                })
                .collect();
            table.insert("eq_band_settings".into(), toml::Value::Array(toml_bands));
        }
    }

    if let Some(rg) = &view.replay_gain {
        let mut rg_table = match table.get("replaygain_settings") {
            Some(toml::Value::Table(t)) => t.clone(),
            _ => toml::map::Map::new(),
        };
        if let Some(m) = &rg.mode {
            rg_table.insert(
                "type".into(),
                toml::Value::Integer(replay_gain_mode_to_int(m)),
            );
        }
        if let Some(v) = rg.preamp {
            rg_table.insert("preamp".into(), toml::Value::Integer(v));
        }
        if let Some(v) = rg.prevent_clipping {
            rg_table.insert("noclip".into(), toml::Value::Boolean(v));
        }
        table.insert("replaygain_settings".into(), toml::Value::Table(rg_table));
    }

    if let Some(tone) = &view.tone {
        if let Some(v) = tone.bass {
            table.insert("bass".into(), toml::Value::Integer(v));
        }
        if let Some(v) = tone.treble {
            table.insert("treble".into(), toml::Value::Integer(v));
        }
        if let Some(v) = tone.balance {
            table.insert("balance".into(), toml::Value::Integer(v));
        }
        if let Some(ch) = &tone.channels {
            table.insert(
                "channel_config".into(),
                toml::Value::Integer(channel_config_to_int(ch)),
            );
        }
    }

    std::fs::write(&path, toml::to_string_pretty(&toml::Value::Table(table))?)?;
    println!("✅ Audio settings pulled to {}", path.display());
    println!("ℹ️  Restart Rockbox to apply the new settings.");

    Ok(())
}

// ── push ──────────────────────────────────────────────────────────────────────

pub async fn push() -> Result<(), Error> {
    let token = load_token()?;
    let path = settings_path();

    if !path.exists() {
        return Err(anyhow!(
            "No settings file at {}. Run `rockboxd settings pull` first.",
            path.display()
        ));
    }

    let table: toml::Table = toml::from_str(&std::fs::read_to_string(&path)?)?;

    let get_int = |k: &str| -> Option<i64> { table.get(k)?.as_integer() };
    let get_bool = |k: &str| -> Option<bool> { table.get(k)?.as_bool() };

    // crossfade
    let crossfade_mode = get_int("crossfade").map(|v| crossfade_mode_from_int(v).to_string());
    let fade_in_delay = get_int("fade_in_delay");
    let fade_in_duration = get_int("fade_in_duration");
    let fade_out_delay = get_int("fade_out_delay");
    let fade_out_duration = get_int("fade_out_duration");
    let fade_out_mix_mode = get_int("fade_out_mixmode").map(|v| mix_mode_from_int(v).to_string());

    let crossfade = (crossfade_mode.is_some()
        || fade_in_delay.is_some()
        || fade_in_duration.is_some()
        || fade_out_delay.is_some()
        || fade_out_duration.is_some()
        || fade_out_mix_mode.is_some())
    .then(|| CrossfadeSettings {
        mode: crossfade_mode,
        fade_in_delay,
        fade_in_duration,
        fade_out_delay,
        fade_out_duration,
        fade_out_mix_mode,
    });

    // equalizer
    let eq_enabled = get_bool("eq_enabled");
    let eq_precut = get_int("eq_precut");
    let eq_bands = table
        .get("eq_band_settings")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|b| {
                    let t = b.as_table()?;
                    Some(EqualizerBand {
                        // TOML `cutoff` → API `frequency`
                        frequency: t.get("cutoff")?.as_integer(),
                        gain: t.get("gain")?.as_integer(),
                        q: t.get("q")?.as_integer(),
                    })
                })
                .collect::<Vec<_>>()
        });

    let equalizer =
        (eq_enabled.is_some() || eq_precut.is_some() || eq_bands.is_some()).then(|| {
            EqualizerSettings {
                enabled: eq_enabled,
                precut: eq_precut,
                bands: eq_bands,
            }
        });

    // replaygain
    let replay_gain = table
        .get("replaygain_settings")
        .and_then(|v| v.as_table())
        .map(|rg| ReplayGainSettings {
            mode: rg
                .get("type")
                .and_then(|v| v.as_integer())
                .map(|v| replay_gain_mode_from_int(v).to_string()),
            preamp: rg.get("preamp").and_then(|v| v.as_integer()),
            prevent_clipping: rg.get("noclip").and_then(|v| v.as_bool()),
        });

    // tone
    let bass = get_int("bass");
    let treble = get_int("treble");
    let balance = get_int("balance");
    let channels = get_int("channel_config").map(|v| channel_config_from_int(v).to_string());

    let tone = (bass.is_some() || treble.is_some() || balance.is_some() || channels.is_some())
        .then(|| ToneSettings {
            bass,
            treble,
            balance,
            channels,
        });

    let mut body = serde_json::Map::new();
    if let Some(cf) = crossfade {
        body.insert("crossfade".into(), serde_json::to_value(cf)?);
    }
    if let Some(eq) = equalizer {
        body.insert("equalizer".into(), serde_json::to_value(eq)?);
    }
    if let Some(rg) = replay_gain {
        body.insert("replayGain".into(), serde_json::to_value(rg)?);
    }
    if let Some(t) = tone {
        body.insert("tone".into(), serde_json::to_value(t)?);
    }

    let res = reqwest::Client::new()
        .post(format!(
            "{}/app.rocksky.rockbox.putAudioSettings",
            XRPC_BASE
        ))
        .header("Authorization", format!("Bearer {}", token))
        .json(&body)
        .send()
        .await?;

    check_auth(&res)?;
    if !res.status().is_success() {
        return Err(anyhow!("API error {}: {}", res.status(), res.text().await?));
    }

    println!("✅ Audio settings pushed from {}", path.display());
    Ok(())
}
