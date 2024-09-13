use std::sync::{mpsc::Sender, Arc, Mutex};

use crate::api::rockbox::v1alpha1::{playback_service_server::PlaybackService, *};
use rockbox_sys::{self as rb, events::RockboxCommand};

pub struct Playback {
    cmd_tx: Arc<Mutex<Sender<RockboxCommand>>>,
}

impl Playback {
    pub fn new(cmd_tx: Arc<Mutex<Sender<RockboxCommand>>>) -> Self {
        Self { cmd_tx }
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
        request: tonic::Request<PauseRequest>,
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
        request: tonic::Request<ResumeRequest>,
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
        request: tonic::Request<NextRequest>,
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
        request: tonic::Request<PreviousRequest>,
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
        request: tonic::Request<StatusRequest>,
    ) -> Result<tonic::Response<StatusResponse>, tonic::Status> {
        let status = rb::playback::status();
        Ok(tonic::Response::new(StatusResponse::default()))
    }

    async fn current_track(
        &self,
        request: tonic::Request<CurrentTrackRequest>,
    ) -> Result<tonic::Response<CurrentTrackResponse>, tonic::Status> {
        let track = rb::playback::current_track();
        Ok(tonic::Response::new(track.into()))
    }

    async fn flush_and_reload_tracks(
        &self,
        request: tonic::Request<FlushAndReloadTracksRequest>,
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
        request: tonic::Request<GetFilePositionRequest>,
    ) -> Result<tonic::Response<GetFilePositionResponse>, tonic::Status> {
        let position = rb::playback::get_file_pos();
        Ok(tonic::Response::new(GetFilePositionResponse { position }))
    }

    async fn hard_stop(
        &self,
        request: tonic::Request<HardStopRequest>,
    ) -> Result<tonic::Response<HardStopResponse>, tonic::Status> {
        self.cmd_tx
            .lock()
            .unwrap()
            .send(RockboxCommand::Stop)
            .map_err(|_| tonic::Status::internal("Failed to send command"))?;
        Ok(tonic::Response::new(HardStopResponse::default()))
    }
}
