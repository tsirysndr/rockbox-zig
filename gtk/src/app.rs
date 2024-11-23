use crate::app;
use crate::ui::window::RbApplicationWindow;
use adw::prelude::*;
use adw::subclass::prelude::*;
use gio::subclass::prelude::ApplicationImpl;
use glib::subclass::types::ObjectSubclass;
use glib::WeakRef;
use gtk::{gdk::Display, gio, glib, CssProvider, STYLE_PROVIDER_PRIORITY_APPLICATION};
use std::cell::OnceCell;

mod imp {

    use super::*;
    pub struct RbApplication {
        pub window: OnceCell<WeakRef<RbApplicationWindow>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for RbApplication {
        const NAME: &'static str = "RockboxApplication";
        type Type = app::RbApplication;
        type ParentType = adw::Application;

        fn new() -> Self {
            Self {
                window: OnceCell::new(),
            }
        }
    }

    impl ObjectImpl for RbApplication {}

    impl ApplicationImpl for RbApplication {
        fn activate(&self) {
            let app = self.obj();
            load_css();

            if let Some(weak_window) = self.window.get() {
                weak_window.upgrade().unwrap().present();
                return;
            }

            let window = app.create_window();
            let _ = self.window.set(window.downgrade());

            app.setup_gactions();
        }

        fn shutdown(&self) {
            self.parent_shutdown();
        }
    }

    impl GtkApplicationImpl for RbApplication {}
    impl AdwApplicationImpl for RbApplication {}
}

glib::wrapper! {
  pub struct RbApplication(ObjectSubclass<imp::RbApplication>)
    @extends gio::Application, gtk::Application, adw::Application,
    @implements gio::ActionMap, gio::ActionGroup;
}

fn load_css() {
    let provider = CssProvider::new();
    provider.load_from_string(include_str!("styles.css"));
    gtk::style_context_add_provider_for_display(
        &Display::default().expect("Error initializing gtk css provider."),
        &provider,
        STYLE_PROVIDER_PRIORITY_APPLICATION,
    );
}

impl RbApplication {
    pub fn run() -> glib::ExitCode {
        // Create new GObject and downcast it into RbApplication
        let app = glib::Object::builder::<RbApplication>()
            .property("application-id", Some("mg.tsirysndr.Rockbox.Devel"))
            .property("flags", gio::ApplicationFlags::empty())
            .property("resource-base-path", Some("/mg/tsirysndr/rockbox"))
            .build();

        // Start running gtk::Application
        app.run()
    }

    pub fn create_window(&self) -> RbApplicationWindow {
        let window = RbApplicationWindow::new();
        self.add_window(&window);

        window.present();
        window
    }

    pub fn setup_gactions(&self) {}
}

impl Default for RbApplication {
    fn default() -> Self {
        println!("Creating default RbApplication");
        println!("{:?}", gio::Application::default());

        gio::Application::default()
            .expect("Could not get default GApplication")
            .downcast()
            .unwrap()
    }
}
