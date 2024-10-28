use rockbox_sys::types::user_settings::{NewGlobalSettings, UserSettings};
use tonic::{Request, Response, Status};

use crate::{
    api::rockbox::v1alpha1::{
        settings_service_server::SettingsService, GetGlobalSettingsRequest,
        GetGlobalSettingsResponse, GetSettingsListRequest, GetSettingsListResponse,
        SaveSettingsRequest, SaveSettingsResponse,
    },
    rockbox_url,
};

#[derive(Default)]
pub struct Settings {
    client: reqwest::Client,
}

impl Settings {
    pub fn new(client: reqwest::Client) -> Self {
        Self { client }
    }
}

#[tonic::async_trait]
impl SettingsService for Settings {
    async fn get_global_settings(
        &self,
        _request: Request<GetGlobalSettingsRequest>,
    ) -> Result<Response<GetGlobalSettingsResponse>, Status> {
        let url = format!("{}/settings", rockbox_url());
        let response = self
            .client
            .get(url)
            .send()
            .await
            .map_err(|e| Status::internal(e.to_string()))?;
        let settings = response
            .json::<UserSettings>()
            .await
            .map_err(|e| Status::internal(e.to_string()))?;
        Ok(Response::new(settings.into()))
    }

    async fn get_settings_list(
        &self,
        _request: Request<GetSettingsListRequest>,
    ) -> Result<Response<GetSettingsListResponse>, Status> {
        let url = format!("{}/settingslist", rockbox_url());
        let _response = self
            .client
            .get(url)
            .send()
            .await
            .map_err(|e| Status::internal(e.to_string()))?;
        //let settings = response.json::<UserSettings>().await?;
        todo!()
    }

    async fn save_settings(
        &self,
        request: Request<SaveSettingsRequest>,
    ) -> Result<Response<SaveSettingsResponse>, Status> {
        let settings = request.into_inner();
        let settings: NewGlobalSettings = settings.into();

        let url = format!("{}/settings", rockbox_url());
        self.client
            .put(url)
            .json(&settings)
            .send()
            .await
            .map_err(|e| Status::internal(e.to_string()))?;
        Ok(Response::new(SaveSettingsResponse::default()))
    }
}
