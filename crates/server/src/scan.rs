use futures_util::StreamExt;
use rockbox_discovery::{discover, CHROMECAST_SERVICE_NAME};
use rockbox_graphql::simplebroker::SimpleBroker;
use rockbox_types::device::{Device, AIRPLAY_DEVICE, AIRPLAY_SERVICE_NAME, UPNP_DLNA_DEVICE};
use std::{
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};

/// Returns the always-present virtual output devices (built-in SDL and
/// Snapcast/FIFO).  Squeezelite clients are discovered dynamically via Slim
/// Protocol HELO tracking, not listed here.
pub fn virtual_devices() -> Vec<Device> {
    vec![
        Device {
            id: "builtin".to_string(),
            name: "Rockbox (Built-in)".to_string(),
            host: "localhost".to_string(),
            ip: "127.0.0.1".to_string(),
            port: 0,
            service: "builtin".to_string(),
            app: "builtin".to_string(),
            is_cast_device: false,
            is_source_device: false,
            ..Default::default()
        },
        Device {
            id: "fifo".to_string(),
            name: "Snapcast (FIFO)".to_string(),
            host: "localhost".to_string(),
            ip: "127.0.0.1".to_string(),
            port: 0,
            service: "fifo".to_string(),
            app: "fifo".to_string(),
            is_cast_device: false,
            is_source_device: false,
            ..Default::default()
        },
    ]
}

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
                let mut device = Device::from(info.clone());
                device.service = "chromecast".to_string();
                device.is_cast_device = true;

                let mut devices = devices.lock().unwrap();
                if devices.iter().any(|d| d.id == device.id) {
                    continue;
                }
                devices.push(device.clone());
                SimpleBroker::<Device>::publish(device);
            }
        });
    });
}

/// Discover AirPlay receivers via mDNS (`_raop._tcp.local.`).
pub fn scan_airplay_devices(devices: Arc<Mutex<Vec<Device>>>) {
    thread::spawn(move || {
        tokio::runtime::Runtime::new().unwrap().block_on(async {
            let services = discover(AIRPLAY_SERVICE_NAME);
            tokio::pin!(services);
            while let Some(info) = services.next().await {
                let mut devices = devices.lock().unwrap();
                if devices
                    .iter()
                    .any(|d| d.id == info.get_fullname().to_owned())
                {
                    continue;
                }
                let mut device = Device::from(info.clone());
                device.service = "airplay".to_string();
                device.app = AIRPLAY_DEVICE.to_string();
                device.is_cast_device = true;
                devices.push(device.clone());
                SimpleBroker::<Device>::publish(device);
            }
        });
    });
}

/// Poll squeezelite clients that have connected via Slim Protocol HELO.
/// Runs in a background thread; adds newly connected clients and removes
/// disconnected ones from the shared device list every 2 seconds.
pub fn scan_squeezelite_clients(devices: Arc<Mutex<Vec<Device>>>) {
    thread::spawn(move || loop {
        let connected = rockbox_slim::get_connected_clients();

        let mut devs = devices.lock().unwrap();

        // Add newly connected clients.
        for client in &connected {
            if !devs.iter().any(|d| d.id == client.id) {
                let device = Device {
                    id: client.id.clone(),
                    name: client.name.clone(),
                    host: client.ip.clone(),
                    ip: client.ip.clone(),
                    port: 3483,
                    service: "squeezelite".to_string(),
                    app: "squeezelite".to_string(),
                    is_cast_device: false,
                    is_source_device: false,
                    ..Default::default()
                };
                tracing::info!("slim: discovered client {} ({})", client.name, client.id);
                SimpleBroker::<Device>::publish(device.clone());
                devs.push(device);
            }
        }

        // Remove clients that have disconnected.
        devs.retain(|d| {
            if d.service != "squeezelite" {
                return true;
            }
            connected.iter().any(|c| c.id == d.id)
        });

        drop(devs);
        thread::sleep(Duration::from_secs(2));
    });
}
