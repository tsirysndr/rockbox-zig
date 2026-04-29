use crate::{
    api::rockbox::v1alpha1::{
        bluetooth_service_server::BluetoothService, BluetoothDevice, ConnectBluetoothDeviceRequest,
        ConnectBluetoothDeviceResponse, DisconnectBluetoothDeviceRequest,
        DisconnectBluetoothDeviceResponse, GetBluetoothDevicesRequest, GetBluetoothDevicesResponse,
        ScanBluetoothRequest, ScanBluetoothResponse,
    },
    rockbox_url,
};
use serde::Deserialize;

#[derive(Deserialize)]
struct BtDevice {
    address: String,
    name: String,
    paired: bool,
    trusted: bool,
    connected: bool,
    rssi: Option<i32>,
}

impl From<BtDevice> for BluetoothDevice {
    fn from(d: BtDevice) -> Self {
        BluetoothDevice {
            address: d.address,
            name: d.name,
            paired: d.paired,
            trusted: d.trusted,
            connected: d.connected,
            rssi: d.rssi,
        }
    }
}

pub struct Bluetooth {
    client: reqwest::Client,
}

impl Bluetooth {
    pub fn new(client: reqwest::Client) -> Self {
        Self { client }
    }
}

#[tonic::async_trait]
impl BluetoothService for Bluetooth {
    async fn scan(
        &self,
        request: tonic::Request<ScanBluetoothRequest>,
    ) -> Result<tonic::Response<ScanBluetoothResponse>, tonic::Status> {
        let timeout_secs = request.into_inner().timeout_secs;
        let url = format!(
            "{}/bluetooth/scan?timeout_secs={}",
            rockbox_url(),
            timeout_secs
        );
        let response = self
            .client
            .post(&url)
            .send()
            .await
            .map_err(|e| tonic::Status::internal(e.to_string()))?;
        let devices = response
            .json::<Vec<BtDevice>>()
            .await
            .map_err(|e| tonic::Status::internal(e.to_string()))?;
        Ok(tonic::Response::new(ScanBluetoothResponse {
            devices: devices.into_iter().map(|d| d.into()).collect(),
        }))
    }

    async fn get_devices(
        &self,
        _request: tonic::Request<GetBluetoothDevicesRequest>,
    ) -> Result<tonic::Response<GetBluetoothDevicesResponse>, tonic::Status> {
        let url = format!("{}/bluetooth/devices", rockbox_url());
        let response = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| tonic::Status::internal(e.to_string()))?;
        let devices = response
            .json::<Vec<BtDevice>>()
            .await
            .map_err(|e| tonic::Status::internal(e.to_string()))?;
        Ok(tonic::Response::new(GetBluetoothDevicesResponse {
            devices: devices.into_iter().map(|d| d.into()).collect(),
        }))
    }

    async fn connect_device(
        &self,
        request: tonic::Request<ConnectBluetoothDeviceRequest>,
    ) -> Result<tonic::Response<ConnectBluetoothDeviceResponse>, tonic::Status> {
        let address = request.into_inner().address;
        let url = format!("{}/bluetooth/devices/{}/connect", rockbox_url(), address);
        self.client
            .put(&url)
            .send()
            .await
            .map_err(|e| tonic::Status::internal(e.to_string()))?;
        Ok(tonic::Response::new(ConnectBluetoothDeviceResponse {}))
    }

    async fn disconnect(
        &self,
        request: tonic::Request<DisconnectBluetoothDeviceRequest>,
    ) -> Result<tonic::Response<DisconnectBluetoothDeviceResponse>, tonic::Status> {
        let address = request.into_inner().address;
        let url = format!("{}/bluetooth/devices/{}/disconnect", rockbox_url(), address);
        self.client
            .put(&url)
            .send()
            .await
            .map_err(|e| tonic::Status::internal(e.to_string()))?;
        Ok(tonic::Response::new(DisconnectBluetoothDeviceResponse {}))
    }
}
