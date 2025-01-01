use std::sync::{mpsc::Sender, Arc, Mutex};

use rockbox_library::{entity::folder::Folder, repo};
use rockbox_sys::{
    events::RockboxCommand,
    types::{playlist_amount::PlaylistAmount, playlist_info::PlaylistInfo},
};
use sqlx::Sqlite;

use crate::{
    api::rockbox::v1alpha1::{playlist_service_server::PlaylistService, *},
    rockbox_url,
    types::StatusCode,
};

pub struct Playlist {
    cmd_tx: Arc<Mutex<Sender<RockboxCommand>>>,
    client: reqwest::Client,
    pool: sqlx::Pool<Sqlite>,
}

impl Playlist {
    pub fn new(
        cmd_tx: Arc<Mutex<Sender<RockboxCommand>>>,
        client: reqwest::Client,
        pool: sqlx::Pool<Sqlite>,
    ) -> Self {
        Self {
            cmd_tx,
            client,
            pool,
        }
    }
}

#[tonic::async_trait]
impl PlaylistService for Playlist {
    async fn get_current(
        &self,
        _request: tonic::Request<GetCurrentRequest>,
    ) -> Result<tonic::Response<GetCurrentResponse>, tonic::Status> {
        let url = format!("{}/playlists/current", rockbox_url());
        let response = self
            .client
            .get(url)
            .send()
            .await
            .map_err(|e| tonic::Status::internal(e.to_string()))?;
        let data = response
            .json::<PlaylistInfo>()
            .await
            .map_err(|e| tonic::Status::internal(e.to_string()))?;
        let tracks = data
            .entries
            .iter()
            .map(|track| CurrentTrackResponse::from(track.clone()))
            .collect::<Vec<CurrentTrackResponse>>();
        Ok(tonic::Response::new(GetCurrentResponse {
            index: data.index,
            amount: data.amount,
            max_playlist_size: data.max_playlist_size,
            first_index: data.first_index,
            last_insert_pos: data.last_insert_pos,
            seed: data.seed,
            last_shuffled_start: data.last_shuffled_start,
            tracks,
        }))
    }

    async fn get_resume_info(
        &self,
        _request: tonic::Request<GetResumeInfoRequest>,
    ) -> Result<tonic::Response<GetResumeInfoResponse>, tonic::Status> {
        Ok(tonic::Response::new(GetResumeInfoResponse::default()))
    }

    async fn get_track_info(
        &self,
        _request: tonic::Request<GetTrackInfoRequest>,
    ) -> Result<tonic::Response<GetTrackInfoResponse>, tonic::Status> {
        Ok(tonic::Response::new(GetTrackInfoResponse::default()))
    }

    async fn get_first_index(
        &self,
        _request: tonic::Request<GetFirstIndexRequest>,
    ) -> Result<tonic::Response<GetFirstIndexResponse>, tonic::Status> {
        Ok(tonic::Response::new(GetFirstIndexResponse::default()))
    }

    async fn get_display_index(
        &self,
        _request: tonic::Request<GetDisplayIndexRequest>,
    ) -> Result<tonic::Response<GetDisplayIndexResponse>, tonic::Status> {
        Ok(tonic::Response::new(GetDisplayIndexResponse::default()))
    }

    async fn amount(
        &self,
        _request: tonic::Request<AmountRequest>,
    ) -> Result<tonic::Response<AmountResponse>, tonic::Status> {
        let url = format!("{}/playlists/amount", rockbox_url());
        let response = self
            .client
            .get(url)
            .send()
            .await
            .map_err(|e| tonic::Status::internal(e.to_string()))?;
        let data = response
            .json::<PlaylistAmount>()
            .await
            .map_err(|e| tonic::Status::internal(e.to_string()))?;
        Ok(tonic::Response::new(AmountResponse {
            amount: data.amount,
        }))
    }

    async fn playlist_resume(
        &self,
        _request: tonic::Request<PlaylistResumeRequest>,
    ) -> Result<tonic::Response<PlaylistResumeResponse>, tonic::Status> {
        let url = format!("{}/playlists/resume", rockbox_url());
        let response = self
            .client
            .put(&url)
            .send()
            .await
            .map_err(|e| tonic::Status::internal(e.to_string()))?;
        let response = response
            .json::<StatusCode>()
            .await
            .map_err(|e| tonic::Status::internal(e.to_string()))?;
        Ok(tonic::Response::new(PlaylistResumeResponse {
            code: response.code,
        }))
    }

    async fn resume_track(
        &self,
        _request: tonic::Request<ResumeTrackRequest>,
    ) -> Result<tonic::Response<ResumeTrackResponse>, tonic::Status> {
        self.cmd_tx
            .lock()
            .unwrap()
            .send(RockboxCommand::PlaylistResumeTrack)
            .map_err(|_| tonic::Status::internal("Failed to send command"))?;
        Ok(tonic::Response::new(ResumeTrackResponse::default()))
    }

    async fn set_modified(
        &self,
        _request: tonic::Request<SetModifiedRequest>,
    ) -> Result<tonic::Response<SetModifiedResponse>, tonic::Status> {
        Ok(tonic::Response::new(SetModifiedResponse::default()))
    }

    async fn start(
        &self,
        request: tonic::Request<StartRequest>,
    ) -> Result<tonic::Response<StartResponse>, tonic::Status> {
        let request = request.into_inner();

        let mut url = format!("{}/playlists/start", rockbox_url());

        if let Some(start_index) = request.start_index {
            url = format!("{}?start_index={}", url, start_index);
        }

        if let Some(elapsed) = request.elapsed {
            url = match url.contains("?") {
                true => format!("{}&elapsed={}", url, elapsed),
                false => format!("{}?elapsed={}", url, elapsed),
            };
        }

        if let Some(offset) = request.offset {
            url = match url.contains("?") {
                true => format!("{}&offset={}", url, offset),
                false => format!("{}?offset={}", url, offset),
            };
        }

        self.client
            .put(&url)
            .send()
            .await
            .map_err(|e| tonic::Status::internal(e.to_string()))?;
        Ok(tonic::Response::new(StartResponse::default()))
    }

    async fn sync(
        &self,
        _request: tonic::Request<SyncRequest>,
    ) -> Result<tonic::Response<SyncResponse>, tonic::Status> {
        Ok(tonic::Response::new(SyncResponse::default()))
    }

    async fn remove_all_tracks(
        &self,
        request: tonic::Request<RemoveAllTracksRequest>,
    ) -> Result<tonic::Response<RemoveAllTracksResponse>, tonic::Status> {
        let request = request.into_inner();
        let playlist_id = request.playlist_id.unwrap_or("current".to_string());
        let body = serde_json::json!({
            "positions": [],
        });
        let url = format!("{}/playlists/{}/tracks", rockbox_url(), playlist_id);
        self.client
            .delete(&url)
            .json(&body)
            .send()
            .await
            .map_err(|e| tonic::Status::internal(e.to_string()))?;
        Ok(tonic::Response::new(RemoveAllTracksResponse::default()))
    }

    async fn remove_tracks(
        &self,
        request: tonic::Request<RemoveTracksRequest>,
    ) -> Result<tonic::Response<RemoveTracksResponse>, tonic::Status> {
        let request = request.into_inner();
        let playlist_id = request.playlist_id.unwrap_or("current".to_string());
        let body = serde_json::json!({
            "positions": request.positions,
        });
        let url = format!("{}/playlists/{}/tracks", rockbox_url(), playlist_id);
        self.client
            .delete(&url)
            .json(&body)
            .send()
            .await
            .map_err(|e| tonic::Status::internal(e.to_string()))?;
        Ok(tonic::Response::new(RemoveTracksResponse::default()))
    }

    async fn create_playlist(
        &self,
        request: tonic::Request<CreatePlaylistRequest>,
    ) -> Result<tonic::Response<CreatePlaylistResponse>, tonic::Status> {
        let request = request.into_inner();
        let body = match request.name {
            Some(name) => serde_json::json!({
                "name": name,
                "tracks": request.tracks,
                "folder_id": request.folder_id,
            }),
            None => serde_json::json!({
                "tracks": request.tracks,
                "folder_id": request.folder_id,
            }),
        };

        let url = format!("{}/playlists", rockbox_url());
        let response = self
            .client
            .post(&url)
            .json(&body)
            .send()
            .await
            .map_err(|e| tonic::Status::internal(e.to_string()))?;
        let start_index = response
            .text()
            .await
            .map_err(|e| tonic::Status::internal(e.to_string()))?
            .parse()
            .unwrap_or(-1);
        Ok(tonic::Response::new(CreatePlaylistResponse { start_index }))
    }

    async fn insert_tracks(
        &self,
        request: tonic::Request<InsertTracksRequest>,
    ) -> Result<tonic::Response<InsertTracksResponse>, tonic::Status> {
        let request = request.into_inner();
        let playlist_id = request.playlist_id.unwrap_or("current".to_string());
        let body = serde_json::json!({
            "position": request.position,
            "tracks": request.tracks,
        });
        let url = format!("{}/playlists/{}/tracks", rockbox_url(), playlist_id);
        let client = reqwest::Client::new();
        client
            .post(&url)
            .json(&body)
            .send()
            .await
            .map_err(|e| tonic::Status::internal(e.to_string()))?;
        Ok(tonic::Response::new(InsertTracksResponse::default()))
    }

    async fn insert_directory(
        &self,
        request: tonic::Request<InsertDirectoryRequest>,
    ) -> Result<tonic::Response<InsertDirectoryResponse>, tonic::Status> {
        let request = request.into_inner();
        let playlist_id = request.playlist_id.unwrap_or("current".to_string());
        let body = serde_json::json!({
            "position": request.position,
            "tracks": [],
            "directory": request.directory,
        });
        let url = format!("{}/playlists/{}/tracks", rockbox_url(), playlist_id);
        self.client
            .post(&url)
            .json(&body)
            .send()
            .await
            .map_err(|e| tonic::Status::internal(e.to_string()))?;
        Ok(tonic::Response::new(InsertDirectoryResponse::default()))
    }

    async fn insert_playlist(
        &self,
        _request: tonic::Request<InsertPlaylistRequest>,
    ) -> Result<tonic::Response<InsertPlaylistResponse>, tonic::Status> {
        Ok(tonic::Response::new(InsertPlaylistResponse::default()))
    }

    async fn shuffle_playlist(
        &self,
        request: tonic::Request<ShufflePlaylistRequest>,
    ) -> Result<tonic::Response<ShufflePlaylistResponse>, tonic::Status> {
        let request = request.into_inner();
        let url = format!(
            "{}/playlists/shuffle?start_index={}",
            rockbox_url(),
            request.start_index
        );
        self.client
            .put(&url)
            .send()
            .await
            .map_err(|e| tonic::Status::internal(e.to_string()))?;
        Ok(tonic::Response::new(ShufflePlaylistResponse::default()))
    }

    async fn insert_album(
        &self,
        request: tonic::Request<InsertAlbumRequest>,
    ) -> Result<tonic::Response<InsertAlbumResponse>, tonic::Status> {
        let request = request.into_inner();
        let album_id = request.album_id;
        let position = request.position;
        let tracks = repo::album_tracks::find_by_album(self.pool.clone(), &album_id)
            .await
            .map_err(|e| tonic::Status::internal(e.to_string()))?;
        let tracks: Vec<String> = tracks.into_iter().map(|t| t.path).collect();
        let body = serde_json::json!({
            "position": position,
            "tracks": tracks,
        });
        let playlist_id = request.playlist_id.unwrap_or("current".to_string());
        let url = format!("{}/playlists/{}/tracks", rockbox_url(), playlist_id);

        self.client
            .post(&url)
            .json(&body)
            .send()
            .await
            .map_err(|e| tonic::Status::internal(e.to_string()))?;

        Ok(tonic::Response::new(InsertAlbumResponse::default()))
    }

    async fn insert_artist_tracks(
        &self,
        request: tonic::Request<InsertArtistTracksRequest>,
    ) -> Result<tonic::Response<InsertArtistTracksResponse>, tonic::Status> {
        let request = request.into_inner();
        let artist_id = request.artist_id;
        let position = request.position;
        let tracks = repo::artist_tracks::find_by_artist(self.pool.clone(), &artist_id)
            .await
            .map_err(|e| tonic::Status::internal(e.to_string()))?;
        let tracks: Vec<String> = tracks.into_iter().map(|t| t.path).collect();
        let body = serde_json::json!({
            "position": position,
            "tracks": tracks,
        });
        let playlist_id = request.playlist_id.unwrap_or("current".to_string());
        let url = format!("{}/playlists/{}/tracks", rockbox_url(), playlist_id);
        self.client
            .post(&url)
            .json(&body)
            .send()
            .await
            .map_err(|e| tonic::Status::internal(e.to_string()))?;

        Ok(tonic::Response::new(InsertArtistTracksResponse::default()))
    }

    async fn get_playlist(
        &self,
        request: tonic::Request<GetPlaylistRequest>,
    ) -> Result<tonic::Response<GetPlaylistResponse>, tonic::Status> {
        let request = request.into_inner();
        let url = format!("{}/playlists/{}", rockbox_url(), request.playlist_id);
        let client = reqwest::Client::new();
        let response = client
            .get(url)
            .send()
            .await
            .map_err(|e| tonic::Status::internal(e.to_string()))?;
        let playlist = response
            .json::<PlaylistInfo>()
            .await
            .map_err(|e| tonic::Status::internal(e.to_string()))?;
        let tracks = playlist
            .entries
            .iter()
            .map(|track| CurrentTrackResponse::from(track.clone()))
            .collect::<Vec<CurrentTrackResponse>>();
        Ok(tonic::Response::new(GetPlaylistResponse {
            id: playlist.id.unwrap_or_default(),
            amount: playlist.amount,
            name: playlist.name.unwrap_or_default(),
            folder_id: playlist.folder_id,
            created_at: playlist.created_at.unwrap_or_default(),
            updated_at: playlist.updated_at.unwrap_or_default(),
            tracks,
            ..Default::default()
        }))
    }
    async fn get_playlists(
        &self,
        request: tonic::Request<GetPlaylistsRequest>,
    ) -> Result<tonic::Response<GetPlaylistsResponse>, tonic::Status> {
        let request = request.into_inner();
        let playlists = repo::playlist::find_by_folder(self.pool.clone(), request.folder_id)
            .await
            .map_err(|e| tonic::Status::internal(e.to_string()))?;
        let playlists = playlists
            .into_iter()
            .map(|playlist| GetPlaylistResponse {
                id: playlist.id,
                name: playlist.name,
                folder_id: playlist.folder_id,
                image: playlist.image,
                description: playlist.description,
                created_at: playlist.created_at.to_rfc3339(),
                updated_at: playlist.updated_at.to_rfc3339(),
                ..Default::default()
            })
            .collect::<Vec<GetPlaylistResponse>>();
        Ok(tonic::Response::new(GetPlaylistsResponse { playlists }))
    }

    async fn create_folder(
        &self,
        request: tonic::Request<CreateFolderRequest>,
    ) -> Result<tonic::Response<CreateFolderResponse>, tonic::Status> {
        let request = request.into_inner();
        let url = format!("{}/folders", rockbox_url());
        let body = match request.parent_id {
            Some(parent_id) => serde_json::json!({
            "name": request.name,
            "parent_id": parent_id,
            }),
            None => serde_json::json!({
                "name": request.name,
            }),
        };
        let client = reqwest::Client::new();
        let response = client
            .post(&url)
            .json(&body)
            .send()
            .await
            .map_err(|e| tonic::Status::internal(e.to_string()))?;
        let response = response
            .json::<Folder>()
            .await
            .map_err(|e| tonic::Status::internal(e.to_string()))?;
        let folder_id = response.id;

        Ok(tonic::Response::new(CreateFolderResponse { folder_id }))
    }

    async fn get_folder(
        &self,
        request: tonic::Request<GetFolderRequest>,
    ) -> Result<tonic::Response<GetFolderResponse>, tonic::Status> {
        let request = request.into_inner();
        let url = format!("{}/folders/{}", rockbox_url(), request.id);
        let client = reqwest::Client::new();
        let response = client
            .get(url)
            .send()
            .await
            .map_err(|e| tonic::Status::internal(e.to_string()))?;
        let folder = response
            .json::<Folder>()
            .await
            .map_err(|e| tonic::Status::internal(e.to_string()))?;
        Ok(tonic::Response::new(GetFolderResponse {
            id: folder.id,
            name: folder.name,
            parent_id: folder.parent_id,
        }))
    }

    async fn get_folders(
        &self,
        request: tonic::Request<GetFoldersRequest>,
    ) -> Result<tonic::Response<GetFoldersResponse>, tonic::Status> {
        let request = request.into_inner();
        let folders = repo::folder::find_by_parent(self.pool.clone(), request.parent_id)
            .await
            .map_err(|e| tonic::Status::internal(e.to_string()))?;
        let folders = folders
            .into_iter()
            .map(|folder| GetFolderResponse {
                id: folder.id,
                name: folder.name,
                parent_id: folder.parent_id,
            })
            .collect::<Vec<GetFolderResponse>>();
        Ok(tonic::Response::new(GetFoldersResponse { folders }))
    }

    async fn remove_folder(
        &self,
        request: tonic::Request<RemoveFolderRequest>,
    ) -> Result<tonic::Response<RemoveFolderResponse>, tonic::Status> {
        let request = request.into_inner();
        let url = format!("{}/folders/{}", rockbox_url(), request.id);
        let client = reqwest::Client::new();
        client
            .delete(&url)
            .send()
            .await
            .map_err(|e| tonic::Status::internal(e.to_string()))?;
        Ok(tonic::Response::new(RemoveFolderResponse::default()))
    }

    async fn remove_playlist(
        &self,
        request: tonic::Request<RemovePlaylistRequest>,
    ) -> Result<tonic::Response<RemovePlaylistResponse>, tonic::Status> {
        let request = request.into_inner();
        let url = format!("{}/playlists/{}", rockbox_url(), request.id);
        let client = reqwest::Client::new();
        client
            .delete(&url)
            .send()
            .await
            .map_err(|e| tonic::Status::internal(e.to_string()))?;
        Ok(tonic::Response::new(RemovePlaylistResponse::default()))
    }

    async fn rename_playlist(
        &self,
        request: tonic::Request<RenamePlaylistRequest>,
    ) -> Result<tonic::Response<RenamePlaylistResponse>, tonic::Status> {
        let request = request.into_inner();
        let url = format!("{}/playlists/{}", rockbox_url(), request.id);
        let client = reqwest::Client::new();
        client
            .put(&url)
            .json(&serde_json::json!({"name": request.name}))
            .send()
            .await
            .map_err(|e| tonic::Status::internal(e.to_string()))?;
        Ok(tonic::Response::new(RenamePlaylistResponse::default()))
    }

    async fn rename_folder(
        &self,
        request: tonic::Request<RenameFolderRequest>,
    ) -> Result<tonic::Response<RenameFolderResponse>, tonic::Status> {
        let request = request.into_inner();
        let url = format!("{}/folders/{}", rockbox_url(), request.id);
        let client = reqwest::Client::new();
        client
            .put(&url)
            .json(&serde_json::json!({"name": request.name}))
            .send()
            .await
            .map_err(|e| tonic::Status::internal(e.to_string()))?;
        Ok(tonic::Response::new(RenameFolderResponse::default()))
    }

    async fn move_playlist(
        &self,
        request: tonic::Request<MovePlaylistRequest>,
    ) -> Result<tonic::Response<MovePlaylistResponse>, tonic::Status> {
        let request = request.into_inner();
        let url = format!("{}/playlists/{}", rockbox_url(), request.playlist_id);
        let client = reqwest::Client::new();
        client
            .put(&url)
            .json(&serde_json::json!({"folder_id": request.folder_id}))
            .send()
            .await
            .map_err(|e| tonic::Status::internal(e.to_string()))?;
        Ok(tonic::Response::new(MovePlaylistResponse::default()))
    }

    async fn move_folder(
        &self,
        request: tonic::Request<MoveFolderRequest>,
    ) -> Result<tonic::Response<MoveFolderResponse>, tonic::Status> {
        let request = request.into_inner();
        let url = format!("{}/folders/{}", rockbox_url(), request.folder_id);
        let client = reqwest::Client::new();
        client
            .put(&url)
            .json(&serde_json::json!({"parent_id": request.parent_id}))
            .send()
            .await
            .map_err(|e| tonic::Status::internal(e.to_string()))?;
        Ok(tonic::Response::new(MoveFolderResponse::default()))
    }
}
