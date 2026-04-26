use futures_util::StreamExt;
use rockbox_discovery::{discover, CHROMECAST_SERVICE_NAME};
use rockbox_graphql::simplebroker::SimpleBroker;
use rockbox_types::device::{Device, UPNP_DLNA_DEVICE};
use std::{
    sync::{Arc, Mutex},
    thread,
};

/// Scan the local network for UPnP/DLNA MediaRenderer devices (Kodi, etc.).
/// Runs once at startup; discovered renderers are added to the shared devices list.
pub fn scan_upnp_devices(devices: Arc<Mutex<Vec<Device>>>) {
    thread::spawn(move || {
        tokio::runtime::Runtime::new().unwrap().block_on(async {
            let renderers = rockbox_upnp::scan::scan_renderers(3).await;
            let mut devices = devices.lock().unwrap();
            for r in renderers {
                let id = if r.udn.is_empty() {
                    format!("{:x}", md5::compute(r.location.as_bytes()))
                } else {
                    r.udn.clone()
                };
                if devices.iter().any(|d| d.id == id) {
                    continue;
                }
                let device = Device {
                    id: id.clone(),
                    name: r.friendly_name.clone(),
                    host: r.ip.clone(),
                    ip: r.ip.clone(),
                    port: r.port,
                    service: "upnp".to_string(),
                    app: UPNP_DLNA_DEVICE.to_string(),
                    base_url: Some(r.av_transport_url.clone()),
                    is_cast_device: true,
                    is_source_device: false,
                    ..Default::default()
                };
                SimpleBroker::<Device>::publish(device.clone());
                devices.push(device);
            }
        });
    });
}

pub fn scan_chromecast_devices(devices: Arc<Mutex<Vec<Device>>>) {
    thread::spawn(move || {
        tokio::runtime::Runtime::new().unwrap().block_on(async {
            let services = discover(CHROMECAST_SERVICE_NAME);
            tokio::pin!(services);
            while let Some(info) = services.next().await {
                let mut devices = devices.lock().unwrap();
                if devices
                    .iter()
                    .any(|d| d.id == info.get_fullname().to_owned())
                {
                    continue;
                }
                devices.push(Device::from(info.clone()));
                SimpleBroker::<Device>::publish(Device::from(info.clone()));
            }
        });
    });
}
