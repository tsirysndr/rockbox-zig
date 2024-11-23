use app::RbApplication;
use gtk::glib;

pub mod app;
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

    RbApplication::run()
}
