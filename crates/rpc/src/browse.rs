use std::{env, fs};

use crate::{
    api::rockbox::v1alpha1::{browse_service_server::BrowseService, *},
    rockbox_url, AUDIO_EXTENSIONS,
};
use rockbox_sys as rb;

#[derive(Default)]
pub struct Browse {
    client: reqwest::Client,
}

impl Browse {
    pub fn new(client: reqwest::Client) -> Self {
        Self { client }
    }
}

#[tonic::async_trait]
impl BrowseService for Browse {
    async fn tree_get_entries(
        &self,
        request: tonic::Request<TreeGetEntriesRequest>,
    ) -> Result<tonic::Response<TreeGetEntriesResponse>, tonic::Status> {
        let path = request.into_inner().path;

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
