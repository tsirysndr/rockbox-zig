pub mod api;
pub mod app;
pub mod client;
pub mod controller;
pub mod http_client;
pub mod state;
pub mod ui;

extern crate env_logger;
extern crate log;

fn main() {
    env_logger::Builder::new()
        .filter_level(log::LevelFilter::Warn)
        .init();
    app::run();
}
