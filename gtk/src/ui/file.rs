use crate::api::rockbox::v1alpha1::browse_service_client::BrowseServiceClient;
use crate::api::rockbox::v1alpha1::playback_service_client::PlaybackServiceClient;
use crate::api::rockbox::v1alpha1::playlist_service_client::PlaylistServiceClient;
use crate::api::rockbox::v1alpha1::{
    InsertDirectoryRequest, InsertTracksRequest, PlayDirectoryRequest, PlayTrackRequest,
    TreeGetEntriesRequest, TreeGetEntriesResponse,
};
use crate::constants::*;
use crate::state::AppState;
use adw::prelude::*;
use adw::subclass::prelude::*;
use anyhow::Error;
use glib::subclass;
use gtk::glib;
use gtk::{CompositeTemplate, Image, Label, ListBox, MenuButton};
use std::cell::{Cell, RefCell};
use std::{env, thread};

mod imp {

    use super::*;

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(file = "./gtk/file.ui")]
    pub struct File {
        #[template_child]
        pub file_icon: TemplateChild<Image>,
        #[template_child]
        pub file_name: TemplateChild<Label>,
        #[template_child]
        pub row: TemplateChild<gtk::Box>,
        #[template_child]
        pub file_menu: TemplateChild<MenuButton>,
        #[template_child]
        pub directory_menu: TemplateChild<MenuButton>,

        pub files: RefCell<Option<ListBox>>,
        pub go_back_button: RefCell<Option<gtk::Button>>,
        pub path: RefCell<String>,
        pub is_dir: Cell<bool>,
        pub state: glib::WeakRef<AppState>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for File {
        const NAME: &'static str = "File";
        type ParentType = gtk::Box;
        type Type = super::File;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);

            klass.install_action("app.dir.play-next", None, move |file, _action, _target| {
                file.play_next();
            });

            klass.install_action("app.dir.play-last", None, move |file, _action, _target| {
                file.play_last();
            });

            klass.install_action(
                "app.dir.add-shuffled",
                None,
                move |file, _action, _target| {
                    file.add_shuffled();
                },
            );

            klass.install_action(
                "app.dir.play-last-shuffled",
                None,
                move |file, _action, _target| {
                    file.play_last_shuffled();
                },
            );

            klass.install_action(
                "app.dir.play-shuffled",
                None,
                move |file, _action, _target| {
                    file.play(true);
                },
            );

            klass.install_action("app.dir.play", None, move |file, _action, _target| {
                file.play(false);
            });
        }

        fn instance_init(obj: &subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for File {
        fn constructed(&self) {
            self.parent_constructed();

            let self_weak = self.downgrade();
            let click = gtk::GestureClick::new();
            click.connect_released(move |_, _, _, _| {
                if let Some(self_) = self_weak.upgrade() {
                    let path = self_.path.borrow();
                    let path = path.clone();
                    let obj = self_.obj();

                    if !self_.is_dir.get() {
                        return;
                    }

                    let state = self_.state.upgrade().unwrap();
                    state.set_current_path(path.clone().as_str());

                    obj.load_files(Some(path));
                    let go_back_button = self_.go_back_button.borrow();
                    if let Some(go_back_button) = go_back_button.as_ref() {
                        go_back_button.set_visible(true);
                    }
                }
            });

            self.row.add_controller(click);

            let self_weak = self.downgrade();
            let gesture = gtk::GestureClick::new();
            let is_dir = self.is_dir.get();
            gesture.connect_pressed(move |gestrure, n_press, _, _| {
                if n_press == 2 && !is_dir {
                    if let Some(self_) = self_weak.upgrade() {
                        let obj = self_.obj();
                        obj.play(false);
                    }
                }
            });
        }
    }

    impl WidgetImpl for File {}
    impl BoxImpl for File {}

    impl File {
        pub fn set_files(&self, files: ListBox) {
            self.files.replace(Some(files));
        }

        pub fn set_go_back_button(&self, go_back_button: Option<gtk::Button>) {
            *self.go_back_button.borrow_mut() = go_back_button;
        }

        pub fn set_path(&self, path: String) {
            *self.path.borrow_mut() = path;
        }

        pub fn set_is_dir(&self, is_dir: bool) {
            self.is_dir.set(is_dir);
            match is_dir {
                true => self.file_icon.set_icon_name(Some("directory-symbolic")),
                false => self.file_icon.set_icon_name(Some("music-alt-symbolic")),
            };
        }
    }
}

glib::wrapper! {
  pub struct File(ObjectSubclass<imp::File>)
    @extends gtk::Widget, gtk::Box;
}

#[gtk::template_callbacks]
impl File {
    pub fn new() -> Self {
        glib::Object::new()
    }

    pub fn load_files(&self, path: Option<String>) {
        let rt = tokio::runtime::Runtime::new().unwrap();
        let response_ = rt.block_on(async {
            let url = build_url();
            let mut client = BrowseServiceClient::connect(url).await?;
            let response = client
                .tree_get_entries(TreeGetEntriesRequest { path: path.clone() })
                .await?
                .into_inner();
            Ok::<TreeGetEntriesResponse, Error>(response)
        });

        if let Ok(response) = response_ {
            let files = self.imp().files.borrow();
            let files_ref = files.as_ref();
            let files_ref = files_ref.unwrap();

            while let Some(file) = files_ref.first_child() {
                files_ref.remove(&file);
            }

            let state = self.imp().state.upgrade().unwrap();

            for entry in response.entries {
                let file = File::new();
                let filename = entry.name.split("/").last().unwrap();
                file.imp().set_files(files_ref.clone());
                file.imp()
                    .set_go_back_button(self.imp().go_back_button.borrow().clone());
                file.imp().file_name.set_text(filename);
                file.imp().state.set(Some(&state));
                file.imp().set_path(entry.name.clone());
                file.imp().set_is_dir(entry.attr == 16);

                match entry.attr == 16 {
                    true => {
                        file.imp().file_menu.set_visible(false);
                        file.imp().directory_menu.set_visible(true);
                    }
                    false => {
                        file.imp().file_menu.set_visible(true);
                        file.imp().directory_menu.set_visible(false);
                    }
                }

                files_ref.append(&file);
            }
        }
    }

    pub fn play_next(&self) {
        let path = self.imp().path.borrow();
        let path = path.clone();
        let is_dir = self.imp().is_dir.get();
        thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new().unwrap();
            let url = build_url();
            let _ = rt.block_on(async {
                let mut client = PlaylistServiceClient::connect(url).await?;
                match is_dir {
                    true => {
                        client
                            .insert_directory(InsertDirectoryRequest {
                                directory: path,
                                position: PLAYLIST_INSERT_FIRST,
                                ..Default::default()
                            })
                            .await?;
                    }
                    false => {
                        client
                            .insert_tracks(InsertTracksRequest {
                                tracks: vec![path],
                                position: PLAYLIST_INSERT_FIRST,
                                ..Default::default()
                            })
                            .await?;
                    }
                }
                Ok::<(), Error>(())
            });
        });
    }

    pub fn play_last(&self) {
        let path = self.imp().path.borrow();
        let path = path.clone();
        let is_dir = self.imp().is_dir.get();
        thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new().unwrap();
            let url = build_url();
            let _ = rt.block_on(async {
                let mut client = PlaylistServiceClient::connect(url).await?;
                match is_dir {
                    true => {
                        client
                            .insert_directory(InsertDirectoryRequest {
                                directory: path,
                                position: PLAYLIST_INSERT_LAST,
                                ..Default::default()
                            })
                            .await?;
                    }
                    false => {
                        client
                            .insert_tracks(InsertTracksRequest {
                                tracks: vec![path],
                                position: PLAYLIST_INSERT_LAST,
                                ..Default::default()
                            })
                            .await?;
                    }
                }
                Ok::<(), Error>(())
            });
        });
    }

    pub fn add_shuffled(&self) {
        let path = self.imp().path.borrow();
        let path = path.clone();
        let is_dir = self.imp().is_dir.get();
        thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new().unwrap();
            let url = build_url();
            let _ = rt.block_on(async {
                let mut client = PlaylistServiceClient::connect(url).await?;
                match is_dir {
                    true => {
                        client
                            .insert_directory(InsertDirectoryRequest {
                                directory: path,
                                position: PLAYLIST_INSERT_SHUFFLED,
                                ..Default::default()
                            })
                            .await?;
                    }
                    false => {
                        client
                            .insert_tracks(InsertTracksRequest {
                                tracks: vec![path],
                                position: PLAYLIST_INSERT_SHUFFLED,
                                ..Default::default()
                            })
                            .await?;
                    }
                }
                Ok::<(), Error>(())
            });
        });
    }

    pub fn play_last_shuffled(&self) {
        let path = self.imp().path.borrow();
        let path = path.clone();
        let is_dir = self.imp().is_dir.get();
        thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new().unwrap();
            let url = build_url();
            let _ = rt.block_on(async {
                let mut client = PlaylistServiceClient::connect(url).await?;
                match is_dir {
                    true => {
                        client
                            .insert_directory(InsertDirectoryRequest {
                                directory: path,
                                position: PLAYLIST_INSERT_LAST_SHUFFLED,
                                ..Default::default()
                            })
                            .await?;
                    }
                    false => {
                        client
                            .insert_tracks(InsertTracksRequest {
                                tracks: vec![path],
                                position: PLAYLIST_INSERT_LAST_SHUFFLED,
                                ..Default::default()
                            })
                            .await?;
                    }
                }
                Ok::<(), Error>(())
            });
        });
    }

    pub fn play(&self, shuffle: bool) {
        let path = self.imp().path.borrow();
        let path = path.clone();
        let is_dir = self.imp().is_dir.get();
        thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new().unwrap();
            let url = build_url();
            let _ = rt.block_on(async {
                let mut client = PlaybackServiceClient::connect(url).await?;
                match is_dir {
                    true => {
                        client
                            .play_directory(PlayDirectoryRequest {
                                path,
                                shuffle: Some(shuffle),
                                recurse: Some(false),
                                ..Default::default()
                            })
                            .await?;
                    }
                    false => {
                        client.play_track(PlayTrackRequest { path }).await?;
                    }
                }
                Ok::<(), Error>(())
            });
        });
    }
}

fn build_url() -> String {
    let host = env::var("ROCKBOX_HOST").unwrap_or("localhost".to_string());
    let port = env::var("ROCKBOX_PORT").unwrap_or("6061".to_string());
    format!("tcp://{}:{}", host, port)
}
