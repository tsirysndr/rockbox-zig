use std::{
    fs,
    pin::Pin,
    sync::{mpsc::Sender, Arc, Mutex},
};

use crate::{
    api::rockbox::v1alpha1::{playback_service_server::PlaybackService, *},
    check_and_load_player, read_files, rockbox_url, AUDIO_EXTENSIONS,
};
use rockbox_library::repo;
use rockbox_sys::{self as rb, events::RockboxCommand, types::audio_status::AudioStatus};
use sqlx::Sqlite;
use tokio_stream::{Stream, StreamExt};
use rockbox_graphql::simplebroker::SimpleBroker;
use rockbox_graphql::schema::objects::track::Track;

pub struct Playback {
    cmd_tx: Arc<Mutex<Sender<RockboxCommand>>>,
    client: reqwest::Client,
    pool: sqlx::Pool<Sqlite>,
}

impl Playback {
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
impl PlaybackService for Playback {
    async fn play(
        &self,
        request: tonic::Request<PlayRequest>,
    ) -> Result<tonic::Response<PlayResponse>, tonic::Status> {
        let params = request.into_inner();
        self.cmd_tx
            .lock()
            .unwrap()
            .send(RockboxCommand::Play(params.elapsed, params.offset))
            .map_err(|_| tonic::Status::internal("Failed to send command"))?;
        Ok(tonic::Response::new(PlayResponse::default()))
    }

    async fn pause(
        &self,
        _request: tonic::Request<PauseRequest>,
    ) -> Result<tonic::Response<PauseResponse>, tonic::Status> {
        self.client
            .put(&format!("{}/player/pause", rockbox_url()))
            .send()
            .await
            .map_err(|e| tonic::Status::internal(e.to_string()))?;
        Ok(tonic::Response::new(PauseResponse::default()))
    }

    async fn resume(
        &self,
        _request: tonic::Request<ResumeRequest>,
    ) -> Result<tonic::Response<ResumeResponse>, tonic::Status> {
        self.client
            .put(&format!("{}/player/resume", rockbox_url()))
            .send()
            .await
            .map_err(|e| tonic::Status::internal(e.to_string()))?;
        Ok(tonic::Response::new(ResumeResponse::default()))
    }

    async fn next(
        &self,
        _request: tonic::Request<NextRequest>,
    ) -> Result<tonic::Response<NextResponse>, tonic::Status> {
        self.client
            .put(&format!("{}/player/next", rockbox_url()))
            .send()
            .await
            .map_err(|e| tonic::Status::internal(e.to_string()))?;
        Ok(tonic::Response::new(NextResponse::default()))
    }

    async fn previous(
        &self,
        _request: tonic::Request<PreviousRequest>,
    ) -> Result<tonic::Response<PreviousResponse>, tonic::Status> {
        self.client
            .put(&format!("{}/player/previous", rockbox_url()))
            .send()
            .await
            .map_err(|e| tonic::Status::internal(e.to_string()))?;
        Ok(tonic::Response::new(PreviousResponse::default()))
    }

    async fn fast_forward_rewind(
        &self,
        request: tonic::Request<FastForwardRewindRequest>,
    ) -> Result<tonic::Response<FastForwardRewindResponse>, tonic::Status> {
        let params = request.into_inner();
        self.cmd_tx
            .lock()
            .unwrap()
            .send(RockboxCommand::FfRewind(params.new_time))
            .map_err(|_| tonic::Status::internal("Failed to send command"))?;
        Ok(tonic::Response::new(FastForwardRewindResponse::default()))
    }

    async fn status(
        &self,
        _request: tonic::Request<StatusRequest>,
    ) -> Result<tonic::Response<StatusResponse>, tonic::Status> {
        let response = self
            .client
            .get(&format!("{}/player/status", rockbox_url()))
            .send()
            .await
            .map_err(|e| tonic::Status::internal(e.to_string()))?
            .json::<AudioStatus>()
            .await
            .map_err(|e| tonic::Status::internal(e.to_string()))?;
        Ok(tonic::Response::new(StatusResponse {
            status: response.status,
        }))
    }

    async fn current_track(
        &self,
        _request: tonic::Request<CurrentTrackRequest>,
    ) -> Result<tonic::Response<CurrentTrackResponse>, tonic::Status> {
        let track = self
            .client
            .get(&format!("{}/player/current-track", rockbox_url()))
            .send()
            .await
            .map_err(|e| tonic::Status::internal(e.to_string()))?
            .json::<Option<rb::types::mp3_entry::Mp3Entry>>()
            .await
            .map_err(|e| tonic::Status::internal(e.to_string()))?;

        if let Some(track) = track.as_ref() {
            let hash = format!("{:x}", md5::compute(track.path.as_bytes()));
            let metadata = repo::track::find_by_md5(self.pool.clone(), &hash)
                .await
                .map_err(|e| tonic::Status::internal(e.to_string()))?;
            if let Some(metadata) = metadata {
                let mut track = track.clone();
                track.album_art = metadata.album_art;
                track.album_id = Some(metadata.album_id);
                track.artist_id = Some(metadata.artist_id);
                return Ok(tonic::Response::new(track.into()));
            }
        }

        Ok(tonic::Response::new(track.into()))
    }

    async fn next_track(
        &self,
        _request: tonic::Request<NextTrackRequest>,
    ) -> Result<tonic::Response<NextTrackResponse>, tonic::Status> {
        let track = self
            .client
            .get(&format!("{}/player/next-track", rockbox_url()))
            .send()
            .await
            .map_err(|e| tonic::Status::internal(e.to_string()))?
            .json::<Option<rb::types::mp3_entry::Mp3Entry>>()
            .await
            .map_err(|e| tonic::Status::internal(e.to_string()))?;
        Ok(tonic::Response::new(track.into()))
    }

    async fn flush_and_reload_tracks(
        &self,
        _request: tonic::Request<FlushAndReloadTracksRequest>,
    ) -> Result<tonic::Response<FlushAndReloadTracksResponse>, tonic::Status> {
        self.cmd_tx
            .lock()
            .unwrap()
            .send(RockboxCommand::FlushAndReloadTracks)
            .map_err(|_| tonic::Status::internal("Failed to send command"))?;
        Ok(tonic::Response::new(FlushAndReloadTracksResponse::default()))
    }

    async fn get_file_position(
        &self,
        _request: tonic::Request<GetFilePositionRequest>,
    ) -> Result<tonic::Response<GetFilePositionResponse>, tonic::Status> {
        let position = self
            .client
            .get(&format!("{}/player/file-position", rockbox_url()))
            .send()
            .await
            .map_err(|e| tonic::Status::internal(e.to_string()))?
            .json::<rb::types::file_position::FilePosition>()
            .await
            .map_err(|e| tonic::Status::internal(e.to_string()))?
            .position;
        Ok(tonic::Response::new(GetFilePositionResponse { position }))
    }

    async fn hard_stop(
        &self,
        _request: tonic::Request<HardStopRequest>,
    ) -> Result<tonic::Response<HardStopResponse>, tonic::Status> {
        self.cmd_tx
            .lock()
            .unwrap()
            .send(RockboxCommand::Stop)
            .map_err(|_| tonic::Status::internal("Failed to send command"))?;
        Ok(tonic::Response::new(HardStopResponse::default()))
    }

    async fn play_album(
        &self,
        request: tonic::Request<PlayAlbumRequest>,
    ) -> Result<tonic::Response<PlayAlbumResponse>, tonic::Status> {
        let request = request.into_inner();
        let album_id = request.album_id;
        let shuffle = request.shuffle;
        let position = request.position;
        let tracks = repo::album_tracks::find_by_album(self.pool.clone(), &album_id)
            .await
            .map_err(|e| tonic::Status::internal(e.to_string()))?;
        let tracks = tracks.into_iter().map(|t| t.path).collect::<Vec<String>>();
        let body = serde_json::json!({
            "tracks": tracks,
        });

        let response = PlayAlbumResponse::default();
        check_and_load_player!(response, tracks, shuffle.unwrap_or_default());

        let url = format!("{}/playlists", rockbox_url());
        self.client
            .post(&url)
            .json(&body)
            .send()
            .await
            .map_err(|e| tonic::Status::internal(e.to_string()))?;

        if let Some(true) = shuffle {
            let url = format!("{}/playlists/shuffle", rockbox_url());
            self.client
                .put(&url)
                .send()
                .await
                .map_err(|e| tonic::Status::internal(e.to_string()))?;
        }

        let url = match position {
            Some(position) => format!("{}/playlists/start?start_index={}", rockbox_url(), position),
            None => format!("{}/playlists/start", rockbox_url()),
        };

        self.client
            .put(&url)
            .send()
            .await
            .map_err(|e| tonic::Status::internal(e.to_string()))?;

        Ok(tonic::Response::new(PlayAlbumResponse::default()))
    }

    async fn play_artist_tracks(
        &self,
        request: tonic::Request<PlayArtistTracksRequest>,
    ) -> Result<tonic::Response<PlayArtistTracksResponse>, tonic::Status> {
        let request = request.into_inner();
        let artist_id = request.artist_id;
        let shuffle = request.shuffle;
        let position = request.position;
        let tracks = repo::artist_tracks::find_by_artist(self.pool.clone(), &artist_id)
            .await
            .map_err(|e| tonic::Status::internal(e.to_string()))?;
        let tracks = tracks.into_iter().map(|t| t.path).collect::<Vec<String>>();
        let body = serde_json::json!({
            "tracks": tracks,
        });

        let response = PlayArtistTracksResponse::default();
        check_and_load_player!(response, tracks, shuffle.unwrap_or_default());

        let url = format!("{}/playlists", rockbox_url());
        self.client
            .post(&url)
            .json(&body)
            .send()
            .await
            .map_err(|e| tonic::Status::internal(e.to_string()))?;

        if let Some(true) = shuffle {
            let url = format!("{}/playlists/shuffle", rockbox_url());
            self.client
                .put(&url)
                .send()
                .await
                .map_err(|e| tonic::Status::internal(e.to_string()))?;
        }

        let url = match position {
            Some(position) => format!("{}/playlists/start?start_index={}", rockbox_url(), position),
            None => format!("{}/playlists/start", rockbox_url()),
        };

        self.client
            .put(&url)
            .send()
            .await
            .map_err(|e| tonic::Status::internal(e.to_string()))?;
        Ok(tonic::Response::new(PlayArtistTracksResponse::default()))
    }

    async fn play_playlist(
        &self,
        _request: tonic::Request<PlayPlaylistRequest>,
    ) -> Result<tonic::Response<PlayPlaylistResponse>, tonic::Status> {
        todo!()
    }

    async fn play_directory(
        &self,
        request: tonic::Request<PlayDirectoryRequest>,
    ) -> Result<tonic::Response<PlayDirectoryResponse>, tonic::Status> {
        let request = request.into_inner();
        let path = request.path;
        let recurse = request.recurse;
        let shuffle = request.shuffle;
        let position = request.position;
        let mut tracks: Vec<String> = vec![];

        let recurse = match position {
            Some(_) => Some(false),
            None => recurse,
        };

        if !std::path::Path::new(&path).is_dir() {
            return Err(tonic::Status::invalid_argument("Path is not a directory"));
        }

        match recurse {
            Some(true) => {
                tracks = read_files(path)
                    .await
                    .map_err(|e| tonic::Status::internal(e.to_string()))?
            }
            _ => {
                for file in
                    fs::read_dir(&path).map_err(|e| tonic::Status::internal(e.to_string()))?
                {
                    let file = file.map_err(|e| tonic::Status::internal(e.to_string()))?;

                    if file
                        .metadata()
                        .map_err(|e| tonic::Status::internal(e.to_string()))?
                        .is_file()
                        && !AUDIO_EXTENSIONS.iter().any(|ext| {
                            file.path()
                                .to_string_lossy()
                                .ends_with(&format!(".{}", ext))
                        })
                    {
                        continue;
                    }

                    tracks.push(file.path().to_string_lossy().to_string());
                }
            }
        }

        let body = serde_json::json!({
            "tracks": tracks,
        });

        let response = PlayDirectoryResponse::default();
        check_and_load_player!(response, tracks, shuffle.unwrap_or_default());

        let url = format!("{}/playlists", rockbox_url());
        self.client
            .post(&url)
            .json(&body)
            .send()
            .await
            .map_err(|e| tonic::Status::internal(e.to_string()))?;

        if let Some(true) = shuffle {
            let url = format!("{}/playlists/shuffle", rockbox_url());
            self.client
                .put(&url)
                .send()
                .await
                .map_err(|e| tonic::Status::internal(e.to_string()))?;
        }

        let url = match position {
            Some(position) => format!("{}/playlists/start?start_index={}", rockbox_url(), position),
            None => format!("{}/playlists/start", rockbox_url()),
        };

        self.client
            .put(&url)
            .send()
            .await
            .map_err(|e| tonic::Status::internal(e.to_string()))?;

        Ok(tonic::Response::new(PlayDirectoryResponse::default()))
    }

    async fn play_track(
        &self,
        request: tonic::Request<PlayTrackRequest>,
    ) -> Result<tonic::Response<PlayTrackResponse>, tonic::Status> {
        let request = request.into_inner();
        let path = request.path.replace("file://", "");
        let tracks = vec![path.clone()];

        let body = serde_json::json!({
            "tracks": tracks,
        });

        let response = PlayTrackResponse::default();
        check_and_load_player!(response, tracks, false);

        let url = format!("{}/playlists", rockbox_url());
        self.client
            .post(&url)
            .json(&body)
            .send()
            .await
            .map_err(|e| tonic::Status::internal(e.to_string()))?;

        let client = reqwest::Client::new();
        let url = format!("{}/playlists/start", rockbox_url());
        client
            .put(&url)
            .send()
            .await
            .map_err(|e| tonic::Status::internal(e.to_string()))?;

        Ok(tonic::Response::new(PlayTrackResponse::default()))
    }

    async fn play_liked_tracks(
        &self,
        request: tonic::Request<PlayLikedTracksRequest>,
    ) -> Result<tonic::Response<PlayLikedTracksResponse>, tonic::Status> {
        let request = request.into_inner();
        let shuffle = request.shuffle;
        let position = request.position;
        let tracks = repo::favourites::all_tracks(self.pool.clone())
            .await
            .map_err(|e| tonic::Status::internal(e.to_string()))?;
        let tracks = tracks.into_iter().map(|t| t.path).collect::<Vec<String>>();
        let body = serde_json::json!({
            "tracks": tracks,
        });

        let response = PlayLikedTracksResponse::default();
        check_and_load_player!(response, tracks, shuffle.unwrap_or_default());

        let url = format!("{}/playlists", rockbox_url());
        self.client
            .post(&url)
            .json(&body)
            .send()
            .await
            .map_err(|e| tonic::Status::internal(e.to_string()))?;

        if let Some(true) = shuffle {
            let url = format!("{}/playlists/shuffle", rockbox_url());
            self.client
                .put(&url)
                .send()
                .await
                .map_err(|e| tonic::Status::internal(e.to_string()))?;
        }

        let url = match position {
            Some(position) => format!("{}/playlists/start?start_index={}", rockbox_url(), position),
            None => format!("{}/playlists/start", rockbox_url()),
        };

        self.client
            .put(&url)
            .send()
            .await
            .map_err(|e| tonic::Status::internal(e.to_string()))?;

        Ok(tonic::Response::new(PlayLikedTracksResponse::default()))
    }

    async fn play_all_tracks(
        &self,
        request: tonic::Request<PlayAllTracksRequest>,
    ) -> Result<tonic::Response<PlayAllTracksResponse>, tonic::Status> {
        let request = request.into_inner();
        let shuffle = request.shuffle;
        let position = request.position;
        let tracks = repo::track::all(self.pool.clone())
            .await
            .map_err(|e| tonic::Status::internal(e.to_string()))?;
        let tracks = tracks.into_iter().map(|t| t.path).collect::<Vec<String>>();
        let body = serde_json::json!({
            "tracks": tracks,
        });

        let response = PlayAllTracksResponse::default();
        check_and_load_player!(response, tracks, shuffle.unwrap_or_default());

        let url = format!("{}/playlists", rockbox_url());
        self.client
            .post(&url)
            .json(&body)
            .send()
            .await
            .map_err(|e| tonic::Status::internal(e.to_string()))?;

        if let Some(true) = shuffle {
            let url = format!("{}/playlists/shuffle", rockbox_url());
            self.client
                .put(&url)
                .send()
                .await
                .map_err(|e| tonic::Status::internal(e.to_string()))?;
        }

        let url = match position {
            Some(position) => format!("{}/playlists/start?start_index={}", rockbox_url(), position),
            None => format!("{}/playlists/start", rockbox_url()),
        };

        self.client
            .put(&url)
            .send()
            .await
            .map_err(|e| tonic::Status::internal(e.to_string()))?;

        Ok(tonic::Response::new(PlayAllTracksResponse::default()))
    }

    type StreamCurrentTrackStream = Pin<
        Box<
            dyn Stream<Item = Result<CurrentTrackResponse, tonic::Status>>
                + Send
                + Sync
                + 'static,
        >,
    >;

    async fn stream_current_track(
        &self,
        _request: tonic::Request<StreamCurrentTrackRequest>,
    ) -> Result<tonic::Response<Self::StreamCurrentTrackStream>, tonic::Status> {
        let mut stream = SimpleBroker::<Track>::subscribe();
        let output = async_stream::try_stream! {
            while let Some(track) = stream.next().await {
                yield track.into();
            }
        };

        Ok(tonic::Response::new(Box::pin(output) as Self::StreamCurrentTrackStream))
    }
}
