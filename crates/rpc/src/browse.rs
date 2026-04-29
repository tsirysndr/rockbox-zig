use std::{env, fs};

use crate::{
    api::rockbox::v1alpha1::{browse_service_server::BrowseService, *},
    AUDIO_EXTENSIONS,
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
