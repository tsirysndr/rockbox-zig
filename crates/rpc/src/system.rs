use rockbox_sys::types::{system_status::SystemStatus, RockboxVersion};
use tonic::{Request, Response, Status};

use crate::{
    api::rockbox::v1alpha1::{
        system_service_server::SystemService, GetGlobalStatusRequest, GetGlobalStatusResponse,
        GetRockboxVersionRequest, GetRockboxVersionResponse,
    },
    rockbox_url,
};

#[derive(Default)]
pub struct System {
    client: reqwest::Client,
}

impl System {
    pub fn new(client: reqwest::Client) -> Self {
        Self { client }
    }
}

#[tonic::async_trait]
impl SystemService for System {
    async fn get_global_status(
        &self,
        _request: Request<GetGlobalStatusRequest>,
    ) -> Result<Response<GetGlobalStatusResponse>, Status> {
        let url = format!("{}/status", rockbox_url());
        let response = self
            .client
            .get(url)
            .send()
            .await
            .map_err(|e| Status::internal(e.to_string()))?;
        let status = response
            .json::<SystemStatus>()
            .await
            .map_err(|e| Status::internal(e.to_string()))?;
        Ok(Response::new(status.into()))
    }

    async fn get_rockbox_version(
        &self,
        _request: Request<GetRockboxVersionRequest>,
    ) -> Result<Response<GetRockboxVersionResponse>, Status> {
        let url = format!("{}/version", rockbox_url());
        let response = self
            .client
            .get(url)
            .send()
            .await
            .map_err(|e| Status::internal(e.to_string()))?;
        let version = response
            .json::<RockboxVersion>()
            .await
            .map_err(|e| Status::internal(e.to_string()))?
            .version;
        Ok(Response::new(GetRockboxVersionResponse { version }))
    }
}
