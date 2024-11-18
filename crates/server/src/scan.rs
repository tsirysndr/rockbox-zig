use futures_util::StreamExt;
use rockbox_discovery::{discover, CHROMECAST_SERVICE_NAME};
use rockbox_graphql::simplebroker::SimpleBroker;
use rockbox_types::device::Device;
use std::{
    sync::{Arc, Mutex},
    thread,
};

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
