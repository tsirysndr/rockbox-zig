use crate::api::rockbox::v1alpha1::playlist_service_client::PlaylistServiceClient;
use crate::api::rockbox::v1alpha1::{
    GetFoldersRequest, GetFoldersResponse, GetPlaylistsRequest, GetPlaylistsResponse,
};
use crate::state::AppState;
use crate::ui::pages::playlist_details::PlaylistDetails;
use crate::ui::playlist::Playlist;
use crate::ui::playlist_folder::PlaylistFolder;
use adw::subclass::prelude::*;
use anyhow::Error;
use glib::subclass;
use gtk::glib;
use gtk::glib::property::PropertySet;
use gtk::prelude::WidgetExt;
use gtk::{Button, CompositeTemplate, ListBox};
use std::cell::RefCell;
use std::env;

mod imp {

    use super::*;

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(resource = "/io/github/tsirysndr/Rockbox/gtk/playlists.ui")]
    pub struct Playlists {
        #[template_child]
        pub playlists: TemplateChild<ListBox>,

        pub main_stack: RefCell<Option<adw::ViewStack>>,
        pub go_back_button: RefCell<Option<Button>>,
        pub state: glib::WeakRef<AppState>,
        pub playlist_details: RefCell<Option<PlaylistDetails>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for Playlists {
        const NAME: &'static str = "Playlists";
        type ParentType = gtk::Box;
        type Type = super::Playlists;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
        }

        fn instance_init(obj: &subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for Playlists {
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
                    obj.load_playlists(None);
                });

                let self_ = match self_weak.upgrade() {
                    Some(self_) => self_,
                    None => return glib::ControlFlow::Continue,
                };

                glib::ControlFlow::Break
            });
        }
    }

    impl WidgetImpl for Playlists {}
    impl BoxImpl for Playlists {}

    impl Playlists {
        pub fn set_go_back_button(&self, go_back_button: Button) {
            *self.go_back_button.borrow_mut() = Some(go_back_button);
        }
    }
}

glib::wrapper! {
  pub struct Playlists(ObjectSubclass<imp::Playlists>)
    @extends gtk::Widget, gtk::Box;
}

#[gtk::template_callbacks]
impl Playlists {
    pub fn new() -> Self {
        glib::Object::new()
    }

    pub fn load_playlists(&self, folder: Option<String>) {
        let rt = tokio::runtime::Runtime::new().unwrap();
        let parent_id = folder.clone();
        let folder_id = folder.clone();
        let response = rt.block_on(async {
            let url = build_url();
            let mut client = PlaylistServiceClient::connect(url).await?;
            let response = client
                .get_folders(GetFoldersRequest { parent_id })
                .await?
                .into_inner();
            Ok::<GetFoldersResponse, Error>(response)
        });

        let playlists = self.imp().playlists.get();

        while let Some(playlist) = playlists.first_child() {
            playlists.remove(&playlist);
        }

        let state = self.imp().state.upgrade().unwrap();

        if let Ok(response) = response {
            for entry in response.folders {
                let folder = PlaylistFolder::new();
                folder.imp().folder_name.set_text(&entry.name);
                folder.imp().state.set(Some(&state));
                folder.imp().folder_id.replace(entry.id.clone());
                folder.imp().parent_id.replace(entry.parent_id.clone());
                folder
                    .imp()
                    .go_back_button
                    .replace(self.imp().go_back_button.borrow().clone());
                folder
                    .imp()
                    .playlists
                    .replace(Some(self.imp().playlists.clone()));
                self.imp().playlists.append(&folder);
            }
        }

        let response = rt.block_on(async {
            let url = build_url();
            let mut client = PlaylistServiceClient::connect(url).await?;
            let response = client
                .get_playlists(GetPlaylistsRequest { folder_id })
                .await?
                .into_inner();
            Ok::<GetPlaylistsResponse, Error>(response)
        });

        if let Ok(response) = response {
            let main_stack = self.imp().main_stack.borrow();
            for entry in response.playlists {
                let playlist = Playlist::new();
                playlist.imp().playlist_name.set_text(&entry.name);
                playlist.imp().state.set(Some(&state));
                playlist
                    .imp()
                    .playlist_details
                    .replace(self.imp().playlist_details.borrow().clone());
                playlist.imp().playlist_id.replace(Some(entry.id.clone()));
                playlist.imp().main_stack.replace(main_stack.clone());
                self.imp().playlists.append(&playlist);
            }
        }
    }

    pub fn go_back(&self) {
        let state = self.imp().state.upgrade().unwrap();
        let parent_folder = state.parent_playlist_folder();

        if parent_folder.is_none() {
            let go_back_button = self.imp().go_back_button.borrow();
            let go_back_button = go_back_button.clone().unwrap();
            go_back_button.set_visible(false);
        }

        self.load_playlists(parent_folder);
    }
}

fn build_url() -> String {
    let host = env::var("ROCKBOX_HOST").unwrap_or("localhost".to_string());
    let port = env::var("ROCKBOX_PORT").unwrap_or("6061".to_string());
    format!("tcp://{}:{}", host, port)
}
