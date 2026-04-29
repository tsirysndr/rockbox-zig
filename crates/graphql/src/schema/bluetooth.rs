use async_graphql::*;

use super::objects::bluetooth_device::BluetoothDevice;

#[derive(Default)]
pub struct BluetoothQuery;

#[Object]
impl BluetoothQuery {
    async fn bluetooth_devices(&self, _ctx: &Context<'_>) -> Result<Vec<BluetoothDevice>, Error> {
        #[cfg(target_os = "linux")]
        {
            let devices = rockbox_bluetooth::get_devices().await?;
            return Ok(devices
                .into_iter()
                .map(|d| BluetoothDevice {
                    address: d.address,
                    name: d.name,
                    paired: d.paired,
                    trusted: d.trusted,
                    connected: d.connected,
                    rssi: d.rssi.map(|r| r as i32),
                })
                .collect());
        }
        #[allow(unreachable_code)]
        Err(Error::new("Bluetooth is only supported on Linux"))
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
        #[cfg(target_os = "linux")]
        {
            let secs = timeout_secs.unwrap_or(10).max(1) as u64;
            let devices = rockbox_bluetooth::scan(secs).await?;
            return Ok(devices
                .into_iter()
                .map(|d| BluetoothDevice {
                    address: d.address,
                    name: d.name,
                    paired: d.paired,
                    trusted: d.trusted,
                    connected: d.connected,
                    rssi: d.rssi.map(|r| r as i32),
                })
                .collect());
        }
        #[allow(unreachable_code)]
        Err(Error::new("Bluetooth is only supported on Linux"))
    }

    async fn bluetooth_connect(&self, _ctx: &Context<'_>, address: String) -> Result<bool, Error> {
        #[cfg(target_os = "linux")]
        {
            rockbox_bluetooth::connect(&address).await?;
            return Ok(true);
        }
        #[allow(unreachable_code)]
        Err(Error::new("Bluetooth is only supported on Linux"))
    }

    async fn bluetooth_disconnect(
        &self,
        _ctx: &Context<'_>,
        address: String,
    ) -> Result<bool, Error> {
        #[cfg(target_os = "linux")]
        {
            rockbox_bluetooth::disconnect(&address).await?;
            return Ok(true);
        }
        #[allow(unreachable_code)]
        Err(Error::new("Bluetooth is only supported on Linux"))
    }
}
