use adw::subclass::prelude::*;
use glib::subclass;
use gtk::{glib, CompositeTemplate};
use std::env;
use adw::prelude::*;

mod imp {

    use super::*;

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(resource = "/io/github/tsirysndr/Rockbox/gtk/new_playlist.ui")]
    pub struct NewPlaylistDialog {}

    #[glib::object_subclass]
    impl ObjectSubclass for NewPlaylistDialog {
        const NAME: &'static str = "NewPlaylistDialog";
        type ParentType = adw::Dialog;
        type Type = super::NewPlaylistDialog;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);

            klass.install_action(
                "app.new_playlist_dialog.create",
                None,
                move |dialog, _action, _target| {
                    dialog.close();
                },
            );
        }

        fn instance_init(obj: &subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for NewPlaylistDialog {
        fn constructed(&self) {
            self.parent_constructed();
        }
    }

    impl WidgetImpl for NewPlaylistDialog {}

    impl AdwDialogImpl for NewPlaylistDialog {}
}

glib::wrapper! {
    pub struct NewPlaylistDialog(ObjectSubclass<imp::NewPlaylistDialog>)
    @extends gtk::Widget, adw::Dialog;
}

impl Default for NewPlaylistDialog {
    fn default() -> Self {
        glib::Object::new()
    }
}

fn build_url() -> String {
    let host = env::var("ROCKBOX_HOST").unwrap_or_else(|_| "localhost".to_string());
    let port = env::var("ROCKBOX_PORT").unwrap_or_else(|_| "6061".to_string());

    format!("tcp://{}:{}", host, port)
}
