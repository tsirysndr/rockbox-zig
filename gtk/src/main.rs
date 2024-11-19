use app::RbApplication;
use gtk::glib;

pub mod app;
pub mod ui;

fn main() -> glib::ExitCode {
    // Initialize GTK
    gtk::init().unwrap_or_else(|_| panic!("Failed to initialize GTK."));

    RbApplication::run()
}
