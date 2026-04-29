use async_graphql::*;
use serde::{Deserialize, Serialize};

#[derive(Default, Clone, Serialize, Deserialize)]
pub struct BluetoothDevice {
    pub address: String,
    pub name: String,
    pub paired: bool,
    pub trusted: bool,
    pub connected: bool,
    pub rssi: Option<i32>,
}

#[Object]
impl BluetoothDevice {
    async fn address(&self) -> &str {
        &self.address
    }

    async fn name(&self) -> &str {
        &self.name
    }

    async fn paired(&self) -> bool {
        self.paired
    }

    async fn trusted(&self) -> bool {
        self.trusted
    }

    async fn connected(&self) -> bool {
        self.connected
    }

    async fn rssi(&self) -> Option<i32> {
        self.rssi
    }
}
