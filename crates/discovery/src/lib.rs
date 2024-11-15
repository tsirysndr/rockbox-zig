use async_stream::stream;
use futures_util::Stream;
use mdns_sd::{ServiceDaemon, ServiceEvent, ServiceInfo};
use std::{env, thread};

pub const ROCKBOX_SERVICE_NAME: &'static str = "_rockbox._tcp.local.";
pub const MUSIC_PLAYER_SERVICE_NAME: &'static str = "_music-player._tcp.local.";
pub const XBMC_SERVICE_NAME: &'static str = "_xbmc-jsonrpc-h._tcp.local.";
pub const CHROMECAST_SERVICE_NAME: &'static str = "_googlecast._tcp.local.";

pub struct MdnsResponder {
    responder: libmdns::Responder,
    svc: Vec<libmdns::Service>,
}

impl MdnsResponder {
    pub fn new() -> Self {
        let responder = libmdns::Responder::new().unwrap();
        Self {
            responder,
            svc: vec![],
        }
    }

    pub fn register_service(&mut self, name: &str, port: u16) {
        let device_name = "rockbox";
        let device_name = format!("device_name={}", device_name);

        self.svc.push(self.responder.register(
            "_rockbox._tcp".to_owned(),
            name.to_owned(),
            port,
            &["path=/", device_name.as_str()],
        ));
    }
}

pub fn register_services() {
    let device_id = "123";
    let http_service = format!("http-{}", device_id);
    let graphql_service = format!("graphql-{}", device_id);
    let grpc_service = format!("grpc-{}", device_id);
    let mpd_service = format!("mpd-{}", device_id);

    thread::spawn(move || {
        let http_port = env::var("ROCKBOX_HTTP_PORT").unwrap_or("6061".to_string());
        let graphql_port = env::var("ROCKBOX_GRAPHQL_PORT").unwrap_or("6062".to_string());
        let grpc_port = env::var("ROCKBOX_PORT").unwrap_or("6063".to_string());
        let mpd_port = env::var("ROCKBOX_MPD_PORT").unwrap_or("6600".to_string());
        let mut responder = MdnsResponder::new();
        responder.register_service(&http_service, http_port.parse::<u16>().unwrap());
        responder.register_service(&graphql_service, graphql_port.parse::<u16>().unwrap());
        responder.register_service(&grpc_service, grpc_port.parse::<u16>().unwrap());
        responder.register_service(&mpd_service, mpd_port.parse::<u16>().unwrap());
        loop {
            ::std::thread::sleep(::std::time::Duration::from_secs(10));
        }
    });
}

pub fn register(name: &str, port: u16) {
    let device_name = env::var("ROCKBOX_DEVICE_NAME").unwrap_or("rockbox".to_string());
    let device_name = format!("device_name={}", device_name);

    let responder = libmdns::Responder::new().unwrap();
    let _svc = responder.register(
        "_rockbox._tcp".to_owned(),
        name.to_owned(),
        port,
        &["path=/", device_name.as_str()],
    );

    loop {
        ::std::thread::sleep(::std::time::Duration::from_secs(10));
    }
}

pub fn discover(service_name: &str) -> impl Stream<Item = ServiceInfo> {
    let mdns = ServiceDaemon::new().unwrap();
    let receiver = mdns.browse(&service_name).expect("Failed to browse");

    stream! {
        while let Ok(event) = receiver.recv() {
            match event {
                ServiceEvent::ServiceResolved(info) => {
                    yield info;
                }
                _ => {}
            }
        }
    }
}
