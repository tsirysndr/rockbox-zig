use adw::prelude::*;
use adw::subclass::prelude::*;
use glib::subclass;
use gtk::{glib, CompositeTemplate};
use std::env;

mod imp {

    use super::*;

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(resource = "/io/github/tsirysndr/Rockbox/gtk/edit_playlist.ui")]
    pub struct EditPlaylistDialog {}

    #[glib::object_subclass]
    impl ObjectSubclass for EditPlaylistDialog {
        const NAME: &'static str = "EditPlaylistDialog";
        type ParentType = adw::Dialog;
        type Type = super::EditPlaylistDialog;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);

            klass.install_action(
                "app.edit_playlist_dialog.save",
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

    impl ObjectImpl for EditPlaylistDialog {
        fn constructed(&self) {
            self.parent_constructed();
        }
    }

    impl WidgetImpl for EditPlaylistDialog {}

    impl AdwDialogImpl for EditPlaylistDialog {}
}

glib::wrapper! {
    pub struct EditPlaylistDialog(ObjectSubclass<imp::EditPlaylistDialog>)
    @extends gtk::Widget, adw::Dialog;
}

impl Default for EditPlaylistDialog {
    fn default() -> Self {
        glib::Object::new()
    }
}

fn build_url() -> String {
    let host = env::var("ROCKBOX_HOST").unwrap_or_else(|_| "localhost".to_string());
    let port = env::var("ROCKBOX_PORT").unwrap_or_else(|_| "6061".to_string());

    format!("tcp://{}:{}", host, port)
}
