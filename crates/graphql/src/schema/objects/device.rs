use async_graphql::*;
use serde::{Deserialize, Serialize};

#[derive(Default, Clone, Serialize, Deserialize)]
pub struct Device {
    pub id: String,
    pub name: String,
    pub host: String,
    pub ip: String,
    pub port: u16,
    pub service: String,
    pub app: String,
    pub is_connected: bool,
    pub base_url: Option<String>,
    pub is_cast_device: bool,
    pub is_source_device: bool,
    pub is_current_device: bool,
}

#[Object]
impl Device {
    async fn id(&self) -> &str {
        &self.id
    }

    async fn name(&self) -> &str {
        &self.name
    }

    async fn host(&self) -> &str {
        &self.host
    }

    async fn ip(&self) -> &str {
        &self.ip
    }

    async fn port(&self) -> i32 {
        self.port as i32
    }

    async fn service(&self) -> &str {
        &self.service
    }

    async fn app(&self) -> &str {
        &self.app
    }

    async fn is_connected(&self) -> bool {
        self.is_connected
    }

    async fn base_url(&self) -> Option<&str> {
        self.base_url.as_deref()
    }

    async fn is_cast_device(&self) -> bool {
        self.is_cast_device
    }

    async fn is_source_device(&self) -> bool {
        self.is_source_device
    }

    async fn is_current_device(&self) -> bool {
        self.is_current_device
    }
}
