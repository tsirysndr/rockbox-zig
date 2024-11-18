use local_ip_addr::get_local_ip_address;
use mdns_sd::ServiceInfo;
use serde::{Deserialize, Serialize};

pub const CHROMECAST_SERVICE_NAME: &str = "_googlecast._tcp.local.";
pub const AIRPLAY_SERVICE_NAME: &str = "_raop._tcp.local.";
pub const ROCKBOX_SERVICE_NAME: &str = "_rockbox._tcp.local.";
pub const XBMC_SERVICE_NAME: &str = "_xbmc-jsonrpc-h._tcp.local.";

pub const AIRPLAY_DEVICE: &str = "AirPlay";
pub const CHROMECAST_DEVICE: &str = "Chromecast";
pub const XBMC_DEVICE: &str = "XBMC";
pub const MUSIC_PLAYER_DEVICE: &str = "MusicPlayer";
pub const UPNP_DLNA_DEVICE: &str = "UPnP/DLNA";
pub const ROCKBOX_DEVICE: &str = "Rockbox";

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

impl Device {
    pub fn with_base_url(&mut self, base_url: Option<String>) -> Self {
        self.base_url = base_url;
        self.clone()
    }
}

impl From<ServiceInfo> for Device {
    fn from(srv: ServiceInfo) -> Self {
        if srv.get_fullname().contains("xbmc") {
            return Self {
                id: srv.get_fullname().to_owned(),
                name: srv
                    .get_fullname()
                    .replace(XBMC_SERVICE_NAME, "")
                    .replace(".", "")
                    .to_owned(),
                host: srv
                    .get_hostname()
                    .split_at(srv.get_hostname().len() - 1)
                    .0
                    .to_owned(),
                ip: srv.get_addresses().iter().next().unwrap().to_string(),
                port: srv.get_port(),
                service: srv.get_fullname().to_owned(),
                app: "xbmc".to_owned(),
                is_connected: false,
                base_url: None,
                is_cast_device: true,
                is_source_device: true,
                is_current_device: false,
            };
        }

        if srv.get_fullname().contains(ROCKBOX_SERVICE_NAME) {
            let device_id = srv
                .get_fullname()
                .replace(ROCKBOX_SERVICE_NAME, "")
                .split("-")
                .collect::<Vec<&str>>()[1]
                .replace(".", "")
                .to_owned();

            let is_current_device = device_id == "device_id"
                && srv.get_fullname().split("-").collect::<Vec<&str>>()[0].to_owned() == "http";

            let mut addresses = srv.get_addresses().iter();
            let mut ip = addresses.next().unwrap().to_string();

            if is_current_device {
                ip = get_local_ip_address().unwrap();
            }

            return Self {
                id: device_id.clone(),
                name: srv
                    .get_properties()
                    .get("device_name")
                    .unwrap_or(&device_id.clone())
                    .to_string(),
                host: srv
                    .get_hostname()
                    .split_at(srv.get_hostname().len() - 1)
                    .0
                    .to_owned(),
                ip,
                port: srv.get_port(),
                service: srv.get_fullname().split("-").collect::<Vec<&str>>()[0].to_owned(),
                app: "rockbox".to_owned(),
                is_connected: false,
                base_url: None,
                is_cast_device: true,
                is_source_device: true,
                is_current_device,
            };
        }

        if srv.get_fullname().contains(CHROMECAST_SERVICE_NAME) {
            return Self {
                id: srv.get_properties().get("id").unwrap().to_owned(),
                name: srv.get_properties().get("fn").unwrap().to_owned(),
                host: srv
                    .get_hostname()
                    .split_at(srv.get_hostname().len() - 1)
                    .0
                    .to_owned(),
                ip: srv.get_addresses().iter().next().unwrap().to_string(),
                port: srv.get_port(),
                service: srv.get_fullname().to_owned(),
                app: "chromecast".to_owned(),
                is_connected: false,
                base_url: None,
                is_cast_device: true,
                is_source_device: false,
                is_current_device: false,
            };
        }

        if srv.get_fullname().contains(AIRPLAY_SERVICE_NAME) {
            let name = srv.get_fullname().split("@").collect::<Vec<&str>>()[1]
                .replace(AIRPLAY_SERVICE_NAME, "")
                .to_owned();
            let name = name.split_at(name.len() - 1).0.to_owned();
            return Self {
                id: srv.get_fullname().to_owned(),
                name,
                host: srv
                    .get_hostname()
                    .split_at(srv.get_hostname().len() - 1)
                    .0
                    .to_owned(),
                ip: srv.get_addresses().iter().next().unwrap().to_string(),
                port: srv.get_port(),
                service: srv.get_fullname().to_owned(),
                app: "airplay".to_owned(),
                is_connected: false,
                base_url: None,
                is_cast_device: true,
                is_source_device: false,
                is_current_device: false,
            };
        }

        Self {
            ..Default::default()
        }
    }
}
