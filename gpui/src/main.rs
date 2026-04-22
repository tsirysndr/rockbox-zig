pub mod app;
pub mod controller;
pub mod state;
pub mod ui;

extern crate env_logger;
extern crate log;

fn main() {
    env_logger::Builder::new()
        .filter_level(log::LevelFilter::Error)
        .init();
    app::run();
}
