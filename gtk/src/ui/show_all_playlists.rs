use adw::subclass::prelude::*;
use glib::subclass;
use gtk::{glib, CompositeTemplate};
use std::env;

mod imp {
    use super::*;

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(resource = "/io/github/tsirysndr/Rockbox/gtk/show_all_playlists.ui")]
    pub struct ShowAllPlaylistsDialog {}

    #[glib::object_subclass]
    impl ObjectSubclass for ShowAllPlaylistsDialog {
        const NAME: &'static str = "ShowAllPlaylistsDialog";
        type ParentType = adw::Dialog;
        type Type = super::ShowAllPlaylistsDialog;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
        }

        fn instance_init(obj: &subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for ShowAllPlaylistsDialog {
        fn constructed(&self) {
            self.parent_constructed();
        }
    }

    impl WidgetImpl for ShowAllPlaylistsDialog {}

    impl AdwDialogImpl for ShowAllPlaylistsDialog {}
}

glib::wrapper! {
    pub struct ShowAllPlaylistsDialog(ObjectSubclass<imp::ShowAllPlaylistsDialog>)
    @extends gtk::Widget, adw::Dialog;
}

impl Default for ShowAllPlaylistsDialog {
    fn default() -> Self {
        glib::Object::new()
    }
}

fn build_url() -> String {
    let host = env::var("ROCKBOX_HOST").unwrap_or_else(|_| "localhost".to_string());
    let port = env::var("ROCKBOX_PORT").unwrap_or_else(|_| "6061".to_string());

    format!("tcp://{}:{}", host, port)
}
