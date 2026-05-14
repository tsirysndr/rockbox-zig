use crate::{
    api::rockbox::v1alpha1::{sound_service_server::SoundService, *},
    rockbox_url,
};

#[derive(Default)]
pub struct Sound {
    client: reqwest::Client,
}

impl Sound {
    pub fn new(client: reqwest::Client) -> Self {
        Self { client }
    }
}

#[tonic::async_trait]
impl SoundService for Sound {
    async fn adjust_volume(
        &self,
        request: tonic::Request<AdjustVolumeRequest>,
    ) -> Result<tonic::Response<AdjustVolumeResponse>, tonic::Status> {
        let request = request.into_inner();
        let body = serde_json::json!({
            "steps": request.steps,
        });
        let url = format!("{}/player/volume", rockbox_url());
        self.client
            .put(&url)
            .json(&body)
            .send()
            .await
            .map_err(|e| tonic::Status::internal(e.to_string()))?;

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
        request: tonic::Request<SoundCurrentRequest>,
    ) -> Result<tonic::Response<SoundCurrentResponse>, tonic::Status> {
        let setting = request.into_inner().setting;
        let value = rockbox_sys::sound::current(setting);
        Ok(tonic::Response::new(SoundCurrentResponse { value }))
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

    async fn set_eq(
        &self,
        request: tonic::Request<SetEqRequest>,
    ) -> Result<tonic::Response<SetEqResponse>, tonic::Status> {
        let req = request.into_inner();
        let body = serde_json::json!({
            "enabled": req.enabled,
            "precut": req.precut,
            "bands": req.bands.iter().map(|b| serde_json::json!({
                "cutoff": b.cutoff,
                "q":      b.q,
                "gain":   b.gain,
            })).collect::<Vec<_>>(),
        });
        let url = format!("{}/player/eq", rockbox_url());
        self.client
            .put(&url)
            .json(&body)
            .send()
            .await
            .map_err(|e| tonic::Status::internal(e.to_string()))?;
        Ok(tonic::Response::new(SetEqResponse::default()))
    }

    async fn set_crossfeed(
        &self,
        request: tonic::Request<SetCrossfeedRequest>,
    ) -> Result<tonic::Response<SetCrossfeedResponse>, tonic::Status> {
        let req = request.into_inner();
        let body = serde_json::json!({
            "type":           req.r#type,
            "direct_gain":    req.direct_gain,
            "cross_gain":     req.cross_gain,
            "hf_attenuation": req.hf_attenuation,
            "hf_cutoff":      req.hf_cutoff,
        });
        let url = format!("{}/player/crossfeed", rockbox_url());
        self.client
            .put(&url)
            .json(&body)
            .send()
            .await
            .map_err(|e| tonic::Status::internal(e.to_string()))?;
        Ok(tonic::Response::new(SetCrossfeedResponse::default()))
    }

    async fn set_dithering(
        &self,
        request: tonic::Request<SetDitheringRequest>,
    ) -> Result<tonic::Response<SetDitheringResponse>, tonic::Status> {
        let req = request.into_inner();
        let body = serde_json::json!({ "enabled": req.enabled });
        let url = format!("{}/player/dithering", rockbox_url());
        self.client
            .put(&url)
            .json(&body)
            .send()
            .await
            .map_err(|e| tonic::Status::internal(e.to_string()))?;
        Ok(tonic::Response::new(SetDitheringResponse::default()))
    }

    async fn set_afr(
        &self,
        request: tonic::Request<SetAfrRequest>,
    ) -> Result<tonic::Response<SetAfrResponse>, tonic::Status> {
        let req = request.into_inner();
        let body = serde_json::json!({ "mode": req.mode });
        let url = format!("{}/player/afr", rockbox_url());
        self.client
            .put(&url)
            .json(&body)
            .send()
            .await
            .map_err(|e| tonic::Status::internal(e.to_string()))?;
        Ok(tonic::Response::new(SetAfrResponse::default()))
    }

    async fn set_pbe(
        &self,
        request: tonic::Request<SetPbeRequest>,
    ) -> Result<tonic::Response<SetPbeResponse>, tonic::Status> {
        let req = request.into_inner();
        let body = serde_json::json!({ "mode": req.mode, "precut": req.precut });
        let url = format!("{}/player/pbe", rockbox_url());
        self.client
            .put(&url)
            .json(&body)
            .send()
            .await
            .map_err(|e| tonic::Status::internal(e.to_string()))?;
        Ok(tonic::Response::new(SetPbeResponse::default()))
    }
}
