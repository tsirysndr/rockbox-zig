use crate::api::rockbox::v1alpha1::{sound_service_server::SoundService, *};

#[derive(Default)]
pub struct Sound;

#[tonic::async_trait]
impl SoundService for Sound {
    async fn adjust_volume(
        &self,
        request: tonic::Request<AdjustVolumeRequest>,
    ) -> Result<tonic::Response<AdjustVolumeResponse>, tonic::Status> {
        Ok(tonic::Response::new(AdjustVolumeResponse::default()))
    }

    async fn sound_set(
        &self,
        _request: tonic::Request<SoundSetRequest>,
    ) -> Result<tonic::Response<SoundSetResponse>, tonic::Status> {
        Ok(tonic::Response::new(SoundSetResponse::default()))
    }

    async fn sound_current(
        &self,
        _request: tonic::Request<SoundCurrentRequest>,
    ) -> Result<tonic::Response<SoundCurrentResponse>, tonic::Status> {
        Ok(tonic::Response::new(SoundCurrentResponse::default()))
    }

    async fn sound_default(
        &self,
        _request: tonic::Request<SoundDefaultRequest>,
    ) -> Result<tonic::Response<SoundDefaultResponse>, tonic::Status> {
        Ok(tonic::Response::new(SoundDefaultResponse::default()))
    }

    async fn sound_min(
        &self,
        _request: tonic::Request<SoundMinRequest>,
    ) -> Result<tonic::Response<SoundMinResponse>, tonic::Status> {
        Ok(tonic::Response::new(SoundMinResponse::default()))
    }

    async fn sound_max(
        &self,
        _request: tonic::Request<SoundMaxRequest>,
    ) -> Result<tonic::Response<SoundMaxResponse>, tonic::Status> {
        Ok(tonic::Response::new(SoundMaxResponse::default()))
    }

    async fn sound_unit(
        &self,
        _request: tonic::Request<SoundUnitRequest>,
    ) -> Result<tonic::Response<SoundUnitResponse>, tonic::Status> {
        Ok(tonic::Response::new(SoundUnitResponse::default()))
    }

    async fn sound_val2_phys(
        &self,
        _request: tonic::Request<SoundVal2PhysRequest>,
    ) -> Result<tonic::Response<SoundVal2PhysResponse>, tonic::Status> {
        Ok(tonic::Response::new(SoundVal2PhysResponse::default()))
    }

    async fn get_pitch(
        &self,
        _request: tonic::Request<GetPitchRequest>,
    ) -> Result<tonic::Response<GetPitchResponse>, tonic::Status> {
        Ok(tonic::Response::new(GetPitchResponse::default()))
    }

    async fn set_pitch(
        &self,
        _request: tonic::Request<SetPitchRequest>,
    ) -> Result<tonic::Response<SetPitchResponse>, tonic::Status> {
        Ok(tonic::Response::new(SetPitchResponse::default()))
    }

    async fn beep_play(
        &self,
        _request: tonic::Request<BeepPlayRequest>,
    ) -> Result<tonic::Response<BeepPlayResponse>, tonic::Status> {
        Ok(tonic::Response::new(BeepPlayResponse::default()))
    }

    async fn pcmbuf_fade(
        &self,
        _request: tonic::Request<PcmbufFadeRequest>,
    ) -> Result<tonic::Response<PcmbufFadeResponse>, tonic::Status> {
        Ok(tonic::Response::new(PcmbufFadeResponse::default()))
    }

    async fn pcmbuf_set_low_latency(
        &self,
        _request: tonic::Request<PcmbufSetLowLatencyRequest>,
    ) -> Result<tonic::Response<PcmbufSetLowLatencyResponse>, tonic::Status> {
        Ok(tonic::Response::new(PcmbufSetLowLatencyResponse::default()))
    }

    async fn system_sound_play(
        &self,
        _request: tonic::Request<SystemSoundPlayRequest>,
    ) -> Result<tonic::Response<SystemSoundPlayResponse>, tonic::Status> {
        Ok(tonic::Response::new(SystemSoundPlayResponse::default()))
    }

    async fn keyclick_click(
        &self,
        _request: tonic::Request<KeyclickClickRequest>,
    ) -> Result<tonic::Response<KeyclickClickResponse>, tonic::Status> {
        Ok(tonic::Response::new(KeyclickClickResponse::default()))
    }
}
