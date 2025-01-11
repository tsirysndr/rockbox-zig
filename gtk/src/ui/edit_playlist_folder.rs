use crate::api::rockbox::v1alpha1::playlist_service_client::PlaylistServiceClient;
use crate::api::rockbox::v1alpha1::RenameFolderRequest;
use crate::state::AppState;
use adw::prelude::*;
use adw::subclass::prelude::*;
use anyhow::Error;
use glib::subclass;
use gtk::{glib, CompositeTemplate};
use std::cell::RefCell;
use std::env;
use std::thread;

mod imp {
    use super::*;

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(resource = "/io/github/tsirysndr/Rockbox/gtk/edit_playlist_folder.ui")]
    pub struct EditPlaylistFolderDialog {
        #[template_child]
        pub name: TemplateChild<adw::EntryRow>,

        pub state: glib::WeakRef<AppState>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for EditPlaylistFolderDialog {
        const NAME: &'static str = "EditPlaylistFolderDialog";
        type ParentType = adw::Dialog;
        type Type = super::EditPlaylistFolderDialog;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);

            klass.install_action(
                "app.edit_playlist_folder_dialog.save",
                None,
                move |dialog, _action, _target| {
                    if dialog.rename_folder().is_ok() {
                        dialog.close();
                    }
                },
            );
        }

        fn instance_init(obj: &subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for EditPlaylistFolderDialog {
        fn constructed(&self) {
            self.parent_constructed();
        }
    }

    impl WidgetImpl for EditPlaylistFolderDialog {}

    impl AdwDialogImpl for EditPlaylistFolderDialog {}
}

glib::wrapper! {
    pub struct EditPlaylistFolderDialog(ObjectSubclass<imp::EditPlaylistFolderDialog>)
    @extends gtk::Widget, adw::Dialog;
}

impl Default for EditPlaylistFolderDialog {
    fn default() -> Self {
        glib::Object::new()
    }
}

#[gtk::template_callbacks]
impl EditPlaylistFolderDialog {
    fn rename_folder(&self) -> Result<(), Error> {
        let state = self.imp().state.upgrade().unwrap();
        let name = self.imp().name.text().trim().to_string();

        if name.is_empty() {
            return Err(anyhow::anyhow!("Name cannot be empty"));
        }

        let id = state.selected_playlist_folder().unwrap();

        thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new().unwrap();
            let url = build_url();
            let result = rt.block_on(async {
                let mut client = PlaylistServiceClient::connect(url).await?;
                client
                    .rename_folder(RenameFolderRequest { id, name })
                    .await?;
                Ok::<_, Error>(())
            });
        });
        Ok(())
    }
}

fn build_url() -> String {
    let host = env::var("ROCKBOX_HOST").unwrap_or_else(|_| "localhost".to_string());
    let port = env::var("ROCKBOX_PORT").unwrap_or_else(|_| "6061".to_string());

    format!("tcp://{}:{}", host, port)
}
