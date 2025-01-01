use adw::prelude::*;
use adw::subclass::prelude::*;
use glib::subclass;
use gtk::{glib, CompositeTemplate};
use std::env;

mod imp {
    use super::*;

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(resource = "/io/github/tsirysndr/Rockbox/gtk/delete_playlist.ui")]
    pub struct DeletePlaylistDialog {}

    #[glib::object_subclass]
    impl ObjectSubclass for DeletePlaylistDialog {
        const NAME: &'static str = "DeletePlaylistDialog";
        type ParentType = adw::Dialog;
        type Type = super::DeletePlaylistDialog;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);

            klass.install_action(
                "app.delete_playlist_dialog.delete",
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

    impl ObjectImpl for DeletePlaylistDialog {
        fn constructed(&self) {
            self.parent_constructed();
        }
    }

    impl WidgetImpl for DeletePlaylistDialog {}

    impl AdwDialogImpl for DeletePlaylistDialog {}
}

glib::wrapper! {
    pub struct DeletePlaylistDialog(ObjectSubclass<imp::DeletePlaylistDialog>)
    @extends gtk::Widget, adw::Dialog;
}

impl Default for DeletePlaylistDialog {
    fn default() -> Self {
        glib::Object::new()
    }
}

fn build_url() -> String {
    let host = env::var("ROCKBOX_HOST").unwrap_or_else(|_| "localhost".to_string());
    let port = env::var("ROCKBOX_PORT").unwrap_or_else(|_| "6061".to_string());

    format!("tcp://{}:{}", host, port)
}
