use app::RbApplication;
use gtk::{gio, glib};

#[rustfmt::skip]
mod config;
pub mod app;
pub mod navigation;
pub mod state;
pub mod time;
pub mod types;
pub mod ui;

pub mod api {
    #[path = ""]
    pub mod rockbox {

        #[path = "rockbox.v1alpha1.rs"]
        pub mod v1alpha1;
    }
}

fn main() -> glib::ExitCode {
    // Initialize GTK
    gtk::init().unwrap_or_else(|_| panic!("Failed to initialize GTK."));

    // Load app resources
    let path = &format!("{}/{}/rockbox.gresource", config::DATADIR, config::PKGNAME,);
    let res = gio::Resource::load(path).expect("Could not load resources");
    gio::resources_register(&res);

    RbApplication::run()
}
