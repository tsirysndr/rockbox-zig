use crate::api::rockbox::v1alpha1::playlist_service_client::PlaylistServiceClient;
use crate::api::rockbox::v1alpha1::CreatePlaylistRequest;
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
    #[template(resource = "/io/github/tsirysndr/Rockbox/gtk/new_playlist.ui")]
    pub struct NewPlaylistDialog {
        #[template_child]
        pub name: TemplateChild<adw::EntryRow>,

        pub state: glib::WeakRef<AppState>,
        pub song_path: RefCell<Option<String>>,
    }

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
                    if dialog.create_playlist().is_ok() {
                        dialog.close();
                    }
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

#[gtk::template_callbacks]
impl NewPlaylistDialog {
    fn create_playlist(&self) -> Result<(), Error> {
        let song_path = self.imp().song_path.borrow();
        let song_path = song_path.as_ref();
        let state = self.imp().state.upgrade().unwrap();
        let folder_id = state.selected_playlist_folder();
        let name = self.imp().name.text().to_string();

        if name.is_empty() {
            return Err(anyhow::anyhow!("Name cannot be empty"));
        }

        let name = Some(name);
        let tracks = match song_path {
            Some(song_path) => vec![song_path.clone()],
            None => vec![],
        };

        thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new().unwrap();
            let url = build_url();
            let result = rt.block_on(async {
                let mut client = PlaylistServiceClient::connect(url).await?;
                client
                    .create_playlist(CreatePlaylistRequest {
                        name,
                        tracks,
                        folder_id,
                    })
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
