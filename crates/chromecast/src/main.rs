use rockbox_chromecast::Chromecast;
use rockbox_types::device::Device;

pub fn main() {
    let player = Chromecast::connect(Device {
        host: "192.168.1.60".into(),
        port: 8009,
        ..Default::default()
    })
    .unwrap();
}
