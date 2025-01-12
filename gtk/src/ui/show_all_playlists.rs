use adw::subclass::prelude::*;
use anyhow::Error;
use glib::subclass;
use gtk::{glib, CompositeTemplate};
use std::env;
use std::thread;

use crate::api::rockbox::v1alpha1::playlist_service_client::PlaylistServiceClient;
use crate::api::rockbox::v1alpha1::GetPlaylistsRequest;

mod imp {
    use super::*;

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(resource = "/io/github/tsirysndr/Rockbox/gtk/show_all_playlists.ui")]
    pub struct ShowAllPlaylistsDialog {
        #[template_child]
        pub search_entry: TemplateChild<gtk::SearchEntry>,
    }

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

            let self_weak = self.downgrade();
            glib::idle_add_local(move || {
                let self_ = match self_weak.upgrade() {
                    Some(self_) => self_,
                    None => return glib::ControlFlow::Continue,
                };

                glib::spawn_future_local(async move {
                    let obj = self_.obj();
                    obj.load_playlists();
                });
                glib::ControlFlow::Break
            });
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

#[gtk::template_callbacks]
impl ShowAllPlaylistsDialog {
    fn load_playlists(&self) {
        thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new().unwrap();
            let url = build_url();
            let result = rt.block_on(async {
                let mut client = PlaylistServiceClient::connect(url).await?;
                client
                    .get_playlists(GetPlaylistsRequest { folder_id: None })
                    .await?;
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
