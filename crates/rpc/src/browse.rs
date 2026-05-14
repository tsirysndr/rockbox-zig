use std::{env, fs};

use crate::{
    api::rockbox::v1alpha1::{browse_service_server::BrowseService, *},
    AUDIO_EXTENSIONS,
};
use rockbox_jellyfin::{
    browse_items as jellyfin_browse_items, discover_jellyfin_servers,
    list_views as jellyfin_list_views, parse_base_url as jellyfin_parse_base_url,
    percent_decode as jellyfin_percent_decode, percent_encode as jellyfin_percent_encode,
};
use rockbox_navidrome::{
    browse_directory as navidrome_browse_directory, list_indexes as navidrome_list_indexes,
    parse_base_url as navidrome_parse_base_url, percent_decode as navidrome_percent_decode,
    percent_encode as navidrome_percent_encode,
};
use rockbox_plex::{
    browse_plex, discover_plex_servers, list_sections as list_plex_sections,
    parse_base_url as plex_parse_base_url, percent_decode as plex_percent_decode,
    percent_encode as plex_percent_encode,
};
use rockbox_upnp::control_point::{
    browse_content_directory, discover_media_servers, percent_decode, percent_encode,
};

#[derive(Default)]
pub struct Browse;

#[tonic::async_trait]
impl BrowseService for Browse {
    async fn tree_get_entries(
        &self,
        request: tonic::Request<TreeGetEntriesRequest>,
    ) -> Result<tonic::Response<TreeGetEntriesResponse>, tonic::Status> {
        let path = request.into_inner().path;

        if let Some(ref p) = path {
            if p.starts_with("upnp://") {
                return handle_upnp(p).await;
            }
            if p.starts_with("plex://") {
                return handle_plex(p).await;
            }
            if p.starts_with("jellyfin://") {
                return handle_jellyfin(p).await;
            }
            if p.starts_with("navidrome://") {
                return handle_navidrome(p).await;
            }
        }

        let show_hidden = false;
        let home = env::var("HOME").unwrap();
        let music_library = env::var("ROCKBOX_LIBRARY").unwrap_or(format!("{}/Music", home));

        let path = match path {
            Some(path) => path,
            None => music_library,
        };

        if !fs::metadata(&path)?.is_dir() {
            return Err(tonic::Status::internal("Path is not a directory"));
        }

        let mut entries = vec![];

        for file in fs::read_dir(&path)? {
            let file = file?;

            if file.metadata()?.is_file()
                && !AUDIO_EXTENSIONS.iter().any(|ext| {
                    file.path()
                        .to_string_lossy()
                        .ends_with(&format!(".{}", ext))
                })
            {
                continue;
            }

            if file.file_name().to_string_lossy().starts_with(".") && !show_hidden {
                continue;
            }

            entries.push(Entry {
                name: file.path().to_string_lossy().to_string(),
                time_write: file
                    .metadata()?
                    .modified()?
                    .duration_since(std::time::SystemTime::UNIX_EPOCH)
                    .map_err(|e| tonic::Status::internal(e.to_string()))?
                    .as_secs() as u32,
                attr: match file.metadata()?.is_dir() {
                    true => 0x10,
                    false => 0,
                },
                ..Default::default()
            });
        }

        Ok(tonic::Response::new(TreeGetEntriesResponse { entries }))
    }
}

async fn handle_plex(path: &str) -> Result<tonic::Response<TreeGetEntriesResponse>, tonic::Status> {
    let rest = path.trim_start_matches("plex://");

    // Discovery: rest is empty or carries a token via "?X-Plex-Token=..."
    if rest.is_empty() || rest.starts_with('?') {
        let token: Option<String> = rest
            .strip_prefix("?X-Plex-Token=")
            .map(|t| t.split('&').next().unwrap_or(t))
            .filter(|t| !t.is_empty())
            .map(|t| t.to_string());
        let servers = discover_plex_servers().await;
        let entries = servers
            .into_iter()
            .map(|s| {
                let base_with_token = match &token {
                    Some(t) => format!("{}?X-Plex-Token={}", s.base_url, t),
                    None => s.base_url.clone(),
                };
                Entry {
                    name: format!("plex://{}", plex_percent_encode(&base_with_token)),
                    attr: 0x10,
                    display_name: Some(s.name),
                    ..Default::default()
                }
            })
            .collect();
        return Ok(tonic::Response::new(TreeGetEntriesResponse { entries }));
    }

    let (base_encoded, api_path_encoded) = match rest.find('/') {
        None => (rest, None),
        Some(i) => (&rest[..i], Some(&rest[i + 1..])),
    };
    let (base_url, token) = plex_parse_base_url(&plex_percent_decode(base_encoded));

    match api_path_encoded {
        None => {
            let sections = list_plex_sections(&base_url, token.as_deref()).await;
            let entries = sections
                .into_iter()
                .map(|e| Entry {
                    name: format!("plex://{}/{}", base_encoded, plex_percent_encode(&e.key)),
                    attr: 0x10,
                    display_name: Some(e.title),
                    ..Default::default()
                })
                .collect();
            Ok(tonic::Response::new(TreeGetEntriesResponse { entries }))
        }
        Some(encoded) => {
            let api_path = plex_percent_decode(encoded);
            let content = browse_plex(&base_url, token.as_deref(), &api_path).await;
            let entries = content
                .into_iter()
                .map(|e| {
                    let name = if e.is_container {
                        format!("plex://{}/{}", base_encoded, plex_percent_encode(&e.key))
                    } else {
                        e.stream_url.unwrap_or_else(|| {
                            format!("plex://{}/{}", base_encoded, plex_percent_encode(&e.key))
                        })
                    };
                    Entry {
                        name,
                        attr: if e.is_container { 0x10 } else { 0 },
                        display_name: Some(e.title),
                        ..Default::default()
                    }
                })
                .collect();
            Ok(tonic::Response::new(TreeGetEntriesResponse { entries }))
        }
    }
}

async fn handle_jellyfin(
    path: &str,
) -> Result<tonic::Response<TreeGetEntriesResponse>, tonic::Status> {
    let rest = path.trim_start_matches("jellyfin://");

    // Discovery: rest is empty or carries credentials via "?X-Jellyfin-Token=...&userId=..."
    if rest.is_empty() || rest.starts_with('?') {
        let token: Option<String> = rest.strip_prefix('?').and_then(|q| {
            q.split('&')
                .find(|p| p.starts_with("X-Jellyfin-Token="))
                .and_then(|p| p.strip_prefix("X-Jellyfin-Token="))
                .filter(|v| !v.is_empty())
                .map(|v| v.to_string())
        });
        let user_id: Option<String> = rest.strip_prefix('?').and_then(|q| {
            q.split('&')
                .find(|p| p.starts_with("userId="))
                .and_then(|p| p.strip_prefix("userId="))
                .filter(|v| !v.is_empty())
                .map(|v| v.to_string())
        });
        let servers = discover_jellyfin_servers().await;
        let entries = servers
            .into_iter()
            .map(|s| {
                let base_with_creds = match (&token, &user_id) {
                    (Some(t), Some(u)) => {
                        format!("{}?X-Jellyfin-Token={}&userId={}", s.base_url, t, u)
                    }
                    (Some(t), None) => format!("{}?X-Jellyfin-Token={}", s.base_url, t),
                    _ => s.base_url.clone(),
                };
                Entry {
                    name: format!("jellyfin://{}", jellyfin_percent_encode(&base_with_creds)),
                    attr: 0x10,
                    display_name: Some(s.name),
                    ..Default::default()
                }
            })
            .collect();
        return Ok(tonic::Response::new(TreeGetEntriesResponse { entries }));
    }

    let (base_encoded, api_path_encoded) = match rest.find('/') {
        None => (rest, None),
        Some(i) => (&rest[..i], Some(&rest[i + 1..])),
    };
    let (base_url, token, user_id) =
        jellyfin_parse_base_url(&jellyfin_percent_decode(base_encoded));
    let token = token.unwrap_or_default();
    let user_id = user_id.unwrap_or_default();

    match api_path_encoded {
        None => {
            let views = jellyfin_list_views(&base_url, &token, &user_id).await;
            let entries = views
                .into_iter()
                .map(|e| Entry {
                    name: format!(
                        "jellyfin://{}/{}",
                        base_encoded,
                        jellyfin_percent_encode(&e.id)
                    ),
                    attr: 0x10,
                    display_name: Some(e.name),
                    ..Default::default()
                })
                .collect();
            Ok(tonic::Response::new(TreeGetEntriesResponse { entries }))
        }
        Some(encoded) => {
            let parent_id = jellyfin_percent_decode(encoded);
            let content = jellyfin_browse_items(&base_url, &token, &user_id, &parent_id).await;
            let entries = content
                .into_iter()
                .map(|e| {
                    let name = if e.is_container {
                        format!(
                            "jellyfin://{}/{}",
                            base_encoded,
                            jellyfin_percent_encode(&e.id)
                        )
                    } else {
                        e.stream_url.unwrap_or_else(|| {
                            format!(
                                "jellyfin://{}/{}",
                                base_encoded,
                                jellyfin_percent_encode(&e.id)
                            )
                        })
                    };
                    Entry {
                        name,
                        attr: if e.is_container { 0x10 } else { 0 },
                        display_name: Some(e.name),
                        ..Default::default()
                    }
                })
                .collect();
            Ok(tonic::Response::new(TreeGetEntriesResponse { entries }))
        }
    }
}

async fn handle_upnp(path: &str) -> Result<tonic::Response<TreeGetEntriesResponse>, tonic::Status> {
    let rest = path.trim_start_matches("upnp://");

    if rest.is_empty() {
        let devices = discover_media_servers().await;
        let entries = devices
            .into_iter()
            .map(|d| Entry {
                name: format!("upnp://{}", percent_encode(&d.control_url)),
                attr: 0x10,
                display_name: Some(d.friendly_name),
                ..Default::default()
            })
            .collect();
        return Ok(tonic::Response::new(TreeGetEntriesResponse { entries }));
    }

    let (ctrl_encoded, object_id) = match rest.find('/') {
        None => (rest, "0"),
        Some(i) => (&rest[..i], &rest[i + 1..]),
    };
    let object_id_decoded = if object_id.is_empty() {
        "0".to_string()
    } else {
        percent_decode(object_id)
    };
    let control_url = percent_decode(ctrl_encoded);

    let content = browse_content_directory(&control_url, &object_id_decoded).await;
    let entries = content
        .into_iter()
        .map(|e| {
            let name = if e.is_container {
                format!("upnp://{}/{}", ctrl_encoded, percent_encode(&e.id))
            } else {
                e.uri.unwrap_or_else(|| {
                    format!("upnp://{}/item/{}", ctrl_encoded, percent_encode(&e.id))
                })
            };
            Entry {
                name,
                attr: if e.is_container { 0x10 } else { 0 },
                display_name: Some(e.title),
                ..Default::default()
            }
        })
        .collect();

    Ok(tonic::Response::new(TreeGetEntriesResponse { entries }))
}

async fn handle_navidrome(
    path: &str,
) -> Result<tonic::Response<TreeGetEntriesResponse>, tonic::Status> {
    let rest = path.trim_start_matches("navidrome://");

    // No discovery: empty path returns an empty list so the GPUI can show the manual-add prompt.
    if rest.is_empty() || rest.starts_with('?') {
        return Ok(tonic::Response::new(TreeGetEntriesResponse {
            entries: vec![],
        }));
    }

    let (base_encoded, dir_id_encoded) = match rest.find('/') {
        None => (rest, None),
        Some(i) => (&rest[..i], Some(&rest[i + 1..])),
    };
    let (base_url, user, token, salt) =
        navidrome_parse_base_url(&navidrome_percent_decode(base_encoded));
    let user = user.unwrap_or_default();
    let token = token.unwrap_or_default();
    let salt = salt.unwrap_or_default();

    let entries = match dir_id_encoded {
        None => {
            let items = navidrome_list_indexes(&base_url, &user, &token, &salt, None).await;
            items
                .into_iter()
                .map(|e| Entry {
                    name: format!(
                        "navidrome://{}/{}",
                        base_encoded,
                        navidrome_percent_encode(&e.id)
                    ),
                    attr: 0x10,
                    display_name: Some(e.name),
                    ..Default::default()
                })
                .collect()
        }
        Some(encoded) => {
            let dir_id = navidrome_percent_decode(encoded);
            let items = navidrome_browse_directory(&base_url, &user, &token, &salt, &dir_id).await;
            items
                .into_iter()
                .map(|e| {
                    let name = if e.is_container {
                        format!(
                            "navidrome://{}/{}",
                            base_encoded,
                            navidrome_percent_encode(&e.id)
                        )
                    } else {
                        e.stream_url.unwrap_or_else(|| {
                            format!(
                                "navidrome://{}/{}",
                                base_encoded,
                                navidrome_percent_encode(&e.id)
                            )
                        })
                    };
                    Entry {
                        name,
                        attr: if e.is_container { 0x10 } else { 0 },
                        display_name: Some(e.name),
                        ..Default::default()
                    }
                })
                .collect()
        }
    };

    Ok(tonic::Response::new(TreeGetEntriesResponse { entries }))
}
