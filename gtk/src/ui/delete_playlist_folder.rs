use crate::api::rockbox::v1alpha1::playlist_service_client::PlaylistServiceClient;
use crate::api::rockbox::v1alpha1::RemoveFolderRequest;
use crate::state::AppState;
use adw::prelude::*;
use adw::subclass::prelude::*;
use anyhow::Error;
use glib::subclass;
use gtk::{glib, CompositeTemplate};
use std::env;
use std::thread;

mod imp {

    use super::*;

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(resource = "/io/github/tsirysndr/Rockbox/gtk/delete_playlist_folder.ui")]
    pub struct DeletePlaylistFolderDialog {
        pub state: glib::WeakRef<AppState>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for DeletePlaylistFolderDialog {
        const NAME: &'static str = "DeletePlaylistFolderDialog";
        type ParentType = adw::Dialog;
        type Type = super::DeletePlaylistFolderDialog;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);

            klass.install_action(
                "app.delete_playlist_folder_dialog.delete",
                None,
                move |dialog, _action, _target| {
                    dialog.delete_folder();
                    dialog.close();
                },
            );
        }

        fn instance_init(obj: &subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for DeletePlaylistFolderDialog {
        fn constructed(&self) {
            self.parent_constructed();
        }
    }

    impl WidgetImpl for DeletePlaylistFolderDialog {}

    impl AdwDialogImpl for DeletePlaylistFolderDialog {}
}

glib::wrapper! {
    pub struct DeletePlaylistFolderDialog(ObjectSubclass<imp::DeletePlaylistFolderDialog>)
    @extends gtk::Widget, adw::Dialog;
}

impl Default for DeletePlaylistFolderDialog {
    fn default() -> Self {
        glib::Object::new()
    }
}

#[gtk::template_callbacks]
impl DeletePlaylistFolderDialog {
    fn delete_folder(&self) {
        let state = self.imp().state.upgrade().unwrap();
        let id = state.selected_playlist_folder().unwrap();
        thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new().unwrap();
            let url = build_url();
            let result = rt.block_on(async {
                let mut client = PlaylistServiceClient::connect(url).await?;
                client.remove_folder(RemoveFolderRequest { id }).await?;
                Ok::<_, Error>(())
            });
        });
    }
}

fn build_url() -> String {
    let host = env::var("ROCKBOX_HOST").unwrap_or_else(|_| "localhost".to_string());
    let port = env::var("ROCKBOX_PORT").unwrap_or_else(|_| "6061".to_string());

    format!("tcp://{}:{}", host, port)
}
