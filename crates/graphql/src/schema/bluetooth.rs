use async_graphql::*;

use crate::rockbox_url;

use super::objects::bluetooth_device::BluetoothDevice;

#[derive(Default)]
pub struct BluetoothQuery;

#[Object]
impl BluetoothQuery {
    async fn bluetooth_devices(&self, _ctx: &Context<'_>) -> Result<Vec<BluetoothDevice>, Error> {
        let client = reqwest::Client::new();
        let url = format!("{}/bluetooth/devices", rockbox_url());
        let response = client.get(&url).send().await?;
        let devices = response.json::<Vec<BluetoothDevice>>().await?;
        Ok(devices)
    }
}

#[derive(Default)]
pub struct BluetoothMutation;

#[Object]
impl BluetoothMutation {
    async fn bluetooth_scan(
        &self,
        _ctx: &Context<'_>,
        timeout_secs: Option<i32>,
    ) -> Result<Vec<BluetoothDevice>, Error> {
        let secs = timeout_secs.unwrap_or(10).max(1);
        let client = reqwest::Client::new();
        let url = format!("{}/bluetooth/scan?timeout_secs={}", rockbox_url(), secs);
        let response = client.post(&url).send().await?;
        let devices = response.json::<Vec<BluetoothDevice>>().await?;
        Ok(devices)
    }

    async fn bluetooth_connect(&self, _ctx: &Context<'_>, address: String) -> Result<bool, Error> {
        let client = reqwest::Client::new();
        let url = format!("{}/bluetooth/devices/{}/connect", rockbox_url(), address);
        client.put(&url).send().await?;
        Ok(true)
    }

    async fn bluetooth_disconnect(
        &self,
        _ctx: &Context<'_>,
        address: String,
    ) -> Result<bool, Error> {
        let client = reqwest::Client::new();
        let url = format!("{}/bluetooth/devices/{}/disconnect", rockbox_url(), address);
        client.put(&url).send().await?;
        Ok(true)
    }
}
