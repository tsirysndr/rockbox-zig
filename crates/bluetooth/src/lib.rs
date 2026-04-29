use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BluetoothDevice {
    pub address: String,
    pub name: String,
    pub paired: bool,
    pub trusted: bool,
    pub connected: bool,
    pub rssi: Option<i16>,
}

#[cfg(target_os = "linux")]
mod imp {
    use super::BluetoothDevice;
    use anyhow::Result;
    use bluer::Address;
    use futures::{pin_mut, StreamExt};
    use std::str::FromStr;
    use std::time::Duration;
    use tracing::warn;

    async fn device_info(adapter: &bluer::Adapter, addr: Address) -> Result<BluetoothDevice> {
        let device = adapter.device(addr)?;
        let name = device.name().await?.unwrap_or_default();
        let paired = device.is_paired().await?;
        let trusted = device.is_trusted().await?;
        let connected = device.is_connected().await?;
        let rssi = device.rssi().await?;
        Ok(BluetoothDevice {
            address: addr.to_string(),
            name,
            paired,
            trusted,
            connected,
            rssi,
        })
    }

    pub async fn scan(timeout_secs: u64) -> Result<Vec<BluetoothDevice>> {
        let session = bluer::Session::new().await?;
        let adapter = session.default_adapter().await?;
        adapter.set_powered(true).await?;

        let secs = if timeout_secs == 0 { 10 } else { timeout_secs };

        {
            let discover = adapter.discover_devices().await?;
            pin_mut!(discover);
            let deadline = tokio::time::sleep(Duration::from_secs(secs));
            tokio::pin!(deadline);
            loop {
                tokio::select! {
                    _ = &mut deadline => break,
                    Some(_) = discover.next() => {}
                    else => break,
                }
            }
        }

        let addrs = adapter.device_addresses().await?;
        let mut devices = Vec::new();
        for addr in addrs {
            match device_info(&adapter, addr).await {
                Ok(d) => devices.push(d),
                Err(e) => warn!("bluetooth: skipping {}: {}", addr, e),
            }
        }
        Ok(devices)
    }

    pub async fn get_devices() -> Result<Vec<BluetoothDevice>> {
        let session = bluer::Session::new().await?;
        let adapter = session.default_adapter().await?;

        let addrs = adapter.device_addresses().await?;
        let mut devices = Vec::new();
        for addr in addrs {
            match device_info(&adapter, addr).await {
                Ok(d) => devices.push(d),
                Err(e) => warn!("bluetooth: skipping {}: {}", addr, e),
            }
        }
        Ok(devices)
    }

    pub async fn connect(address: &str) -> Result<()> {
        let session = bluer::Session::new().await?;
        let adapter = session.default_adapter().await?;
        adapter.set_powered(true).await?;

        let addr = Address::from_str(address)?;
        let device = adapter.device(addr)?;

        if !device.is_paired().await? {
            device.pair().await?;
        }
        if !device.is_trusted().await? {
            device.set_trusted(true).await?;
        }
        device.connect().await?;
        Ok(())
    }

    pub async fn disconnect(address: &str) -> Result<()> {
        let session = bluer::Session::new().await?;
        let adapter = session.default_adapter().await?;

        let addr = Address::from_str(address)?;
        let device = adapter.device(addr)?;
        device.disconnect().await?;
        Ok(())
    }
}

#[cfg(target_os = "linux")]
pub use imp::{connect, disconnect, get_devices, scan};

#[doc(hidden)]
pub fn _link_bluetooth() {}
