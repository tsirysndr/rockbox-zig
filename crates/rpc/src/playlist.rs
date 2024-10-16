use std::sync::{mpsc::Sender, Arc, Mutex};

use rockbox_sys::{
    events::RockboxCommand,
    types::{playlist_amount::PlaylistAmount, playlist_info::PlaylistInfo},
};

use crate::{
    api::rockbox::v1alpha1::{playlist_service_server::PlaylistService, *},
    rockbox_url,
};

pub struct Playlist {
    cmd_tx: Arc<Mutex<Sender<RockboxCommand>>>,
    client: reqwest::Client,
}

impl Playlist {
    pub fn new(cmd_tx: Arc<Mutex<Sender<RockboxCommand>>>, client: reqwest::Client) -> Self {
        Self { cmd_tx, client }
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
        self.cmd_tx
            .lock()
            .unwrap()
            .send(RockboxCommand::PlaylistResume)
            .map_err(|_| tonic::Status::internal("Failed to send command"))?;
        Ok(tonic::Response::new(PlaylistResumeResponse::default()))
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
        _request: tonic::Request<RemoveAllTracksRequest>,
    ) -> Result<tonic::Response<RemoveAllTracksResponse>, tonic::Status> {
        let body = serde_json::json!({
            "positions": [],
        });
        let url = format!("{}/playlists/current/tracks", rockbox_url());
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
        let body = serde_json::json!({
            "positions": request.positions,
        });
        let url = format!("{}/playlists/current/tracks", rockbox_url());
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
        let body = serde_json::json!({
            "name": request.name,
            "tracks": request.tracks,
        });

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
        let body = serde_json::json!({
            "position": request.position,
            "tracks": request.tracks,
        });
        let url = format!("{}/playlists/current/tracks", rockbox_url());
        self.client
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
        let body = serde_json::json!({
            "position": request.position,
            "tracks": [],
            "directory": request.directory,
        });
        let url = format!("{}/playlists/current/tracks", rockbox_url());
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
}
