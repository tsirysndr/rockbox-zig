use crate::state::AppState;
use crate::ui::window::{self, RbApplicationWindow};
use crate::{app, ui::media_controls};
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
        pub state: OnceCell<AppState>,
        pub window: OnceCell<WeakRef<RbApplicationWindow>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for RbApplication {
        const NAME: &'static str = "RockboxApplication";
        type Type = app::RbApplication;
        type ParentType = adw::Application;

        fn new() -> Self {
            Self {
                state: OnceCell::new(),
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
            .property("application-id", Some("io.github.tsirysndr.Rockbox"))
            .property("flags", gio::ApplicationFlags::empty())
            .property("resource-base-path", Some("/io/github/tsirysndr/Rockbox"))
            .build();

        // Start running gtk::Application
        app.run()
    }

    pub fn create_window(&self) -> RbApplicationWindow {
        let app_state = self.imp().state.get_or_init(AppState::new);
        let window = RbApplicationWindow::new(app_state.clone());

        self.add_window(&window);

        window.present();
        window
    }

    pub fn setup_gactions(&self) {
        let window = self.imp().window.get();
        let window = window.unwrap().upgrade().unwrap();

        let media_controls = window.imp().media_control_bar.get();
        let like = gio::ActionEntry::builder("like")
            .activate(move |_, _, _| {
                media_controls.like();
            })
            .build();

        let media_controls = window.imp().media_control_bar.get();
        let play = gio::ActionEntry::builder("play_pause")
            .activate(move |_, _, _| {
                media_controls.play();
            })
            .build();

        let media_controls = window.imp().media_control_bar.get();
        let previous = gio::ActionEntry::builder("previous")
            .activate(move |_, _, _| {
                media_controls.previous();
            })
            .build();

        let media_controls = window.imp().media_control_bar.get();
        let next = gio::ActionEntry::builder("next")
            .activate(move |_, _, _| {
                media_controls.next();
            })
            .build();

        let media_controls = window.imp().media_control_bar.get();
        let shuffle = gio::ActionEntry::builder("shuffle")
            .activate(move |_, _, _| {
                media_controls.shuffle();
            })
            .build();

        let window_clone = window.clone();
        let refresh_library = gio::ActionEntry::builder("refresh_library")
            .activate(move |_, _, _| {
                window_clone.imp().refresh_library();
            })
            .build();

        let media_controls = window.imp().media_control_bar.get();
        let seek_backward = gio::ActionEntry::builder("seek_backward")
            .activate(move |_, _, _| {
                media_controls.seek_backward();
            })
            .build();

        let media_controls = window.imp().media_control_bar.get();
        let seek_forward = gio::ActionEntry::builder("seek_forward")
            .activate(move |_, _, _| {
                media_controls.seek_forward();
            })
            .build();

        self.add_action_entries([
            like,
            play,
            previous,
            next,
            shuffle,
            refresh_library,
            seek_backward,
            seek_forward,
        ]);

        self.set_accels_for_action("win.toggle_search", &["<primary>f"]);
        self.set_accels_for_action("app.preferences", &["<primary>comma"]);
        self.set_accels_for_action("app.quit", &["<primary>q"]);

        self.set_accels_for_action("app.refresh_library", &["<primary>r"]);
        self.set_accels_for_action("app.like", &["<primary>l"]);

        self.set_accels_for_action("app.play_pause", &["<primary>space"]);
        self.set_accels_for_action("app.previous", &["<primary>Left"]);
        self.set_accels_for_action("app.next", &["<primary>Right"]);
        self.set_accels_for_action("app.seek_backward", &["<Shift><primary>Left"]);
        self.set_accels_for_action("app.seek_forward", &["<Shift><primary>Right"]);
        self.set_accels_for_action("app.shuffle", &["<primary>s"]);
    }
}

impl Default for RbApplication {
    fn default() -> Self {
        gio::Application::default()
            .expect("Could not get default GApplication")
            .downcast()
            .unwrap()
    }
}
