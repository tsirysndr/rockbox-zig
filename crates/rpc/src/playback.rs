use crate::api::rockbox::v1alpha1::{playback_service_server::PlaybackService, *};
use rockbox_sys as rb;
use tokio::task;

#[derive(Default)]
pub struct Playback;

#[tonic::async_trait]
impl PlaybackService for Playback {
    async fn play(
        &self,
        request: tonic::Request<PlayRequest>,
    ) -> Result<tonic::Response<PlayResponse>, tonic::Status> {
        let params = request.into_inner();
        rb::playback::play(params.elapsed, params.offset);
        Ok(tonic::Response::new(PlayResponse::default()))
    }

    async fn pause(
        &self,
        request: tonic::Request<PauseRequest>,
    ) -> Result<tonic::Response<PauseResponse>, tonic::Status> {
        rb::playback::pause();
        Ok(tonic::Response::new(PauseResponse::default()))
    }

    async fn resume(
        &self,
        request: tonic::Request<ResumeRequest>,
    ) -> Result<tonic::Response<ResumeResponse>, tonic::Status> {
        rb::playback::resume();
        Ok(tonic::Response::new(ResumeResponse::default()))
    }

    async fn next(
        &self,
        request: tonic::Request<NextRequest>,
    ) -> Result<tonic::Response<NextResponse>, tonic::Status> {
        rb::playback::next();
        Ok(tonic::Response::new(NextResponse::default()))
    }

    async fn previous(
        &self,
        request: tonic::Request<PreviousRequest>,
    ) -> Result<tonic::Response<PreviousResponse>, tonic::Status> {
        rb::playback::prev();
        Ok(tonic::Response::new(PreviousResponse::default()))
    }

    async fn fast_forward_rewind(
        &self,
        request: tonic::Request<FastForwardRewindRequest>,
    ) -> Result<tonic::Response<FastForwardRewindResponse>, tonic::Status> {
        let params = request.into_inner();
        rb::playback::ff_rewind(params.new_time);
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
        rb::playback::flush_and_reload_tracks();
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
        rb::playback::hard_stop();
        Ok(tonic::Response::new(HardStopResponse::default()))
    }
}
