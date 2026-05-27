use crate::ui::components::NdSavedServer;

fn config_path() -> Option<std::path::PathBuf> {
    let home = std::env::var("HOME").ok()?;
    let dir = std::path::PathBuf::from(home)
        .join(".config")
        .join("rockbox.org");
    std::fs::create_dir_all(&dir).ok()?;
    Some(dir.join("navidrome_servers.json"))
}

#[derive(serde::Serialize, serde::Deserialize, Default)]
struct PersistedState {
    servers: Vec<NdSavedServer>,
    active_id: Option<String>,
}

pub fn load_servers() -> (Vec<NdSavedServer>, Option<String>) {
    let path = match config_path() {
        Some(p) => p,
        None => return (vec![], None),
    };
    let bytes = match std::fs::read(&path) {
        Ok(b) => b,
        Err(_) => return (vec![], None),
    };
    let state: PersistedState = match serde_json::from_slice(&bytes) {
        Ok(s) => s,
        Err(_) => return (vec![], None),
    };
    (state.servers, state.active_id)
}

pub fn save_servers(servers: &[NdSavedServer], active_id: Option<&str>) {
    let path = match config_path() {
        Some(p) => p,
        None => return,
    };
    let state = PersistedState {
        servers: servers.to_vec(),
        active_id: active_id.map(|s| s.to_string()),
    };
    if let Ok(json) = serde_json::to_vec_pretty(&state) {
        let _ = std::fs::write(&path, json);
    }
}
