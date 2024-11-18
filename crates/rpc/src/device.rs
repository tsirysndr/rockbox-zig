use crate::{
    api::rockbox::v1alpha1::{
        device_service_server::DeviceService, ConnectDeviceRequest, ConnectDeviceResponse,
        DisconnectDeviceRequest, DisconnectDeviceResponse, GetDeviceRequest, GetDeviceResponse,
        GetDevicesRequest, GetDevicesResponse,
    },
    rockbox_url,
};

pub struct Device {
    client: reqwest::Client,
}

impl Device {
    pub fn new(client: reqwest::Client) -> Self {
        Self { client }
    }
}

#[tonic::async_trait]
impl DeviceService for Device {
    async fn get_devices(
        &self,
        _request: tonic::Request<GetDevicesRequest>,
    ) -> Result<tonic::Response<GetDevicesResponse>, tonic::Status> {
        let url = format!("{}/devices", rockbox_url());
        let response = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| tonic::Status::internal(e.to_string()))?;
        let response = response
            .json::<Vec<rockbox_types::device::Device>>()
            .await
            .map_err(|e| tonic::Status::internal(e.to_string()))?;
        Ok(tonic::Response::new(GetDevicesResponse {
            devices: response.into_iter().map(|d| d.into()).collect(),
        }))
    }

    async fn get_device(
        &self,
        request: tonic::Request<GetDeviceRequest>,
    ) -> Result<tonic::Response<GetDeviceResponse>, tonic::Status> {
        let id = request.into_inner().id;
        let url = format!("{}/devices/{}", rockbox_url(), id);
        let response = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| tonic::Status::internal(e.to_string()))?;

        if response.status() == 404 {
            return Ok(tonic::Response::new(GetDeviceResponse { device: None }));
        }

        let response = response
            .json::<Option<rockbox_types::device::Device>>()
            .await
            .map_err(|e| tonic::Status::internal(e.to_string()))?;
        Ok(tonic::Response::new(GetDeviceResponse {
            device: response.map(|d| d.into()),
        }))
    }

    async fn connect_device(
        &self,
        request: tonic::Request<ConnectDeviceRequest>,
    ) -> Result<tonic::Response<ConnectDeviceResponse>, tonic::Status> {
        let id = request.into_inner().id;
        let url = format!("{}/devices/{}/connect", rockbox_url(), id);
        self.client
            .put(&url)
            .send()
            .await
            .map_err(|e| tonic::Status::internal(e.to_string()))?;
        Ok(tonic::Response::new(ConnectDeviceResponse::default()))
    }

    async fn disconnect_device(
        &self,
        request: tonic::Request<DisconnectDeviceRequest>,
    ) -> Result<tonic::Response<DisconnectDeviceResponse>, tonic::Status> {
        let id = request.into_inner().id;
        let url = format!("{}/devices/{}/disconnect", rockbox_url(), id);
        self.client
            .put(&url)
            .send()
            .await
            .map_err(|e| tonic::Status::internal(e.to_string()))?;
        Ok(tonic::Response::new(DisconnectDeviceResponse::default()))
    }
}
