use crate::api::rockbox::v1alpha1::playlist_service_client::PlaylistServiceClient;
use crate::api::rockbox::v1alpha1::{
    GetFoldersRequest, GetFoldersResponse, GetPlaylistsRequest, GetPlaylistsResponse,
};
use crate::state::AppState;
use crate::ui::edit_playlist_folder::EditPlaylistFolderDialog;
use crate::ui::playlist::Playlist;
use crate::ui::{
    delete_playlist_folder::DeletePlaylistFolderDialog, new_playlist::NewPlaylistDialog,
};
use adw::prelude::*;
use adw::subclass::prelude::*;
use anyhow::Error;
use glib::subclass;
use gtk::glib;
use gtk::prelude::WidgetExt;
use gtk::{Button, CompositeTemplate, Image, Label, ListBox, MenuButton};
use std::cell::RefCell;
use std::env;

mod imp {
    use std::borrow::Borrow;

    use super::*;

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(resource = "/io/github/tsirysndr/Rockbox/gtk/playlist_folder.ui")]
    pub struct PlaylistFolder {
        #[template_child]
        pub folder_icon: TemplateChild<Image>,
        #[template_child]
        pub folder_name: TemplateChild<Label>,
        #[template_child]
        pub row: TemplateChild<gtk::Box>,
        #[template_child]
        pub folder_menu: TemplateChild<MenuButton>,

        pub playlists: RefCell<Option<ListBox>>,
        pub main_stack: RefCell<Option<adw::ViewStack>>,
        pub go_back_button: RefCell<Option<Button>>,
        pub folder_id: RefCell<String>,
        pub parent_id: RefCell<Option<String>>,
        pub state: glib::WeakRef<AppState>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for PlaylistFolder {
        const NAME: &'static str = "PlaylistFolder";
        type ParentType = gtk::Box;
        type Type = super::PlaylistFolder;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);

            klass.install_action(
                "app.folder.rename",
                None,
                move |folder, _action, _target| {
                    let edit_playlist_folder_dialog = EditPlaylistFolderDialog::default();
                    let self_ = imp::PlaylistFolder::from_obj(folder);
                    let state = self_.state.upgrade().unwrap();
                    let folder_id = self_.folder_id.borrow();
                    let folder_id = folder_id.clone();
                    state.set_selected_playlist_folder(Some(folder_id));
                    edit_playlist_folder_dialog.imp().state.set(Some(&state));
                    edit_playlist_folder_dialog.present(Some(folder));
                },
            );

            klass.install_action(
                "app.folder.delete",
                None,
                move |folder, _action, _target| {
                    let delete_playlist_folder_dialog = DeletePlaylistFolderDialog::default();
                    let self_ = imp::PlaylistFolder::from_obj(folder);
                    let state = self_.state.upgrade().unwrap();
                    let folder_id = self_.folder_id.borrow();
                    let folder_id = folder_id.clone();
                    state.set_selected_playlist_folder(Some(folder_id));
                    delete_playlist_folder_dialog.imp().state.set(Some(&state));
                    delete_playlist_folder_dialog.present(Some(folder));
                },
            );

            klass.install_action(
                "app.folder.create-playlist",
                None,
                move |folder, _action, _target| {
                    let new_playlist_dialog = NewPlaylistDialog::default();
                    let self_ = imp::PlaylistFolder::from_obj(folder);
                    let state = self_.state.upgrade().unwrap();
                    let folder_id = self_.folder_id.borrow();
                    let folder_id = folder_id.clone();
                    state.set_selected_playlist_folder(Some(folder_id));
                    new_playlist_dialog.imp().state.set(Some(&state));
                    new_playlist_dialog.present(Some(folder));
                },
            );
        }

        fn instance_init(obj: &subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for PlaylistFolder {
        fn constructed(&self) {
            self.parent_constructed();

            let self_weak = self.downgrade();
            let click = gtk::GestureClick::new();
            click.connect_released(move |_, _, _, _| {
                if let Some(self_) = self_weak.upgrade() {
                    let folder_id = self_.folder_id.borrow();
                    let parent_id = self_.parent_id.borrow();
                    let folder_id = folder_id.clone();
                    let obj = self_.obj();

                    let state = self_.state.upgrade().unwrap();
                    state.set_current_playlist_folder(folder_id.clone().as_str());
                    state.set_parent_playlist_folder(parent_id.clone());

                    obj.load_playlists(Some(folder_id.clone()));
                    state.set_selected_playlist_folder(Some(folder_id));

                    let go_back_button = self_.go_back_button.borrow();
                    if let Some(go_back_button) = go_back_button.as_ref() {
                        go_back_button.set_visible(true);
                    }
                }
            });

            self.row.add_controller(click);
        }
    }

    impl WidgetImpl for PlaylistFolder {}
    impl BoxImpl for PlaylistFolder {}
}

glib::wrapper! {
  pub struct PlaylistFolder(ObjectSubclass<imp::PlaylistFolder>)
    @extends gtk::Widget, gtk::Box;
}

#[gtk::template_callbacks]
impl PlaylistFolder {
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

        let playlists = self.imp().playlists.borrow();
        let playlists_ref = playlists.as_ref();
        let playlists_ref = playlists_ref.unwrap();

        while let Some(playlist) = playlists_ref.first_child() {
            playlists_ref.remove(&playlist);
        }

        let state = self.imp().state.upgrade().unwrap();

        if let Ok(response) = response {
            for entry in response.folders {
                let folder = PlaylistFolder::new();
                folder.imp().folder_name.set_text(&entry.name);
                folder.imp().state.set(Some(&state));
                folder.imp().folder_id.replace(entry.id.clone());
                folder.imp().parent_id.replace(entry.parent_id.clone());
                playlists_ref.append(&folder);
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
            for entry in response.playlists {
                let playlist = Playlist::new();
                playlist.imp().playlist_name.set_text(&entry.name);
                playlist.imp().state.set(Some(&state));
                playlists_ref.append(&playlist);
            }
        }
    }
}

fn build_url() -> String {
    let host = env::var("ROCKBOX_HOST").unwrap_or("localhost".to_string());
    let port = env::var("ROCKBOX_PORT").unwrap_or("6061".to_string());
    format!("tcp://{}:{}", host, port)
}
