use std::sync::{mpsc::Sender, Arc, Mutex};

use crate::{
    api::rockbox::v1alpha1::{playback_service_server::PlaybackService, *},
    rockbox_url,
};
use rockbox_sys::{self as rb, events::RockboxCommand, types::audio_status::AudioStatus};

pub struct Playback {
    cmd_tx: Arc<Mutex<Sender<RockboxCommand>>>,
    client: reqwest::Client,
}

impl Playback {
    pub fn new(cmd_tx: Arc<Mutex<Sender<RockboxCommand>>>, client: reqwest::Client) -> Self {
        Self { cmd_tx, client }
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
        self.cmd_tx
            .lock()
            .unwrap()
            .send(RockboxCommand::Pause)
            .map_err(|_| tonic::Status::internal("Failed to send command"))?;
        Ok(tonic::Response::new(PauseResponse::default()))
    }

    async fn resume(
        &self,
        _request: tonic::Request<ResumeRequest>,
    ) -> Result<tonic::Response<ResumeResponse>, tonic::Status> {
        self.cmd_tx
            .lock()
            .unwrap()
            .send(RockboxCommand::Resume)
            .map_err(|_| tonic::Status::internal("Failed to send command"))?;
        Ok(tonic::Response::new(ResumeResponse::default()))
    }

    async fn next(
        &self,
        _request: tonic::Request<NextRequest>,
    ) -> Result<tonic::Response<NextResponse>, tonic::Status> {
        self.cmd_tx
            .lock()
            .unwrap()
            .send(RockboxCommand::Next)
            .map_err(|_| tonic::Status::internal("Failed to send command"))?;
        Ok(tonic::Response::new(NextResponse::default()))
    }

    async fn previous(
        &self,
        _request: tonic::Request<PreviousRequest>,
    ) -> Result<tonic::Response<PreviousResponse>, tonic::Status> {
        self.cmd_tx
            .lock()
            .unwrap()
            .send(RockboxCommand::Prev)
            .map_err(|_| tonic::Status::internal("Failed to send command"))?;
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
            .get(&format!("{}/audio_status", rockbox_url()))
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
}
