use crate::api::rockbox::v1alpha1::library_service_client::LibraryServiceClient;
use crate::api::rockbox::v1alpha1::playback_service_client::PlaybackServiceClient;
use crate::api::rockbox::v1alpha1::playlist_service_client::PlaylistServiceClient;
use crate::api::rockbox::v1alpha1::{
    Album, GetAlbumRequest, InsertAlbumRequest, PlayAlbumRequest, Track,
};
use crate::constants::*;
use crate::state::AppState;
use crate::ui::album_tracks::AlbumTracks;
use adw::prelude::*;
use adw::subclass::prelude::*;
use anyhow::Error;
use glib::subclass;
use gtk::glib;
use gtk::{CompositeTemplate, Image, Label};
use std::cell::RefCell;
use std::{env, thread};

mod imp {

    use super::*;

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(resource = "/mg/tsirysndr/Rockbox/gtk/album_details.ui")]
    pub struct AlbumDetails {
        #[template_child]
        pub album_cover: TemplateChild<Image>,
        #[template_child]
        pub album_title: TemplateChild<Label>,
        #[template_child]
        pub album_artist: TemplateChild<Label>,
        #[template_child]
        pub album_tracks: TemplateChild<Label>,
        #[template_child]
        pub album_year: TemplateChild<Label>,
        #[template_child]
        pub album_tracklist: TemplateChild<gtk::Box>,

        pub state: glib::WeakRef<AppState>,
        pub album_id: RefCell<String>,
        pub play_all_button: RefCell<Option<gtk::Button>>,
        pub shuffle_all_button: RefCell<Option<gtk::Button>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for AlbumDetails {
        const NAME: &'static str = "AlbumDetails";
        type ParentType = gtk::Box;
        type Type = super::AlbumDetails;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);

            klass.install_action(
                "app.play-album",
                None,
                move |album_details, _action, _target| {
                    album_details.play_album(false);
                },
            );

            klass.install_action(
                "app.shuffle-album",
                None,
                move |album_details, _action, _target| {
                    album_details.play_album(true);
                },
            );

            klass.install_action(
                "app.album.play-next",
                None,
                move |album_details, _action, _target| {
                    album_details.play_next();
                },
            );

            klass.install_action(
                "app.album.play-last",
                None,
                move |album_details, _action, _target| {
                    album_details.play_last();
                },
            );

            klass.install_action(
                "app.album.add-shuffled",
                None,
                move |album_details, _action, _target| {
                    album_details.add_shuffled();
                },
            );

            klass.install_action(
                "app.album.play-last-shuffled",
                None,
                move |album_details, _action, _target| {
                    album_details.play_last_shuffled();
                },
            );

            klass.install_action(
                "app.album.play-shuffled",
                None,
                move |album_details, _action, _target| {
                    album_details.play_album(true);
                },
            );
        }

        fn instance_init(obj: &subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for AlbumDetails {
        fn constructed(&self) {
            self.parent_constructed();
        }
    }

    impl WidgetImpl for AlbumDetails {}
    impl BoxImpl for AlbumDetails {}

    impl AlbumDetails {
        pub fn hide_top_buttons(&self, hide: bool) {
            let play_all_button = self.play_all_button.borrow();
            let play_all_button_ref = play_all_button.as_ref();
            let shuffle_all_button = self.shuffle_all_button.borrow();
            let shuffle_all_button_ref = shuffle_all_button.as_ref();
            play_all_button_ref.unwrap().set_visible(!hide);
            shuffle_all_button_ref.unwrap().set_visible(!hide);
        }

        pub fn load_album(&self, id: &str) {
            let id = id.to_string();
            self.album_id.replace(id.clone());
            let handle = thread::spawn(move || {
                let rt = tokio::runtime::Runtime::new().unwrap();
                rt.block_on(async {
                    let url = build_url();
                    let mut client = LibraryServiceClient::connect(url).await?;
                    let response = client.get_album(GetAlbumRequest { id }).await?.into_inner();
                    Ok::<Option<Album>, Error>(response.album)
                })
            });

            if let Ok(Ok(Some(album))) = handle.join() {
                self.album_title.set_text(&album.title);
                self.album_artist.set_text(&album.artist);
                self.album_year.set_text(&format!("{}", album.year));
                self.album_tracks
                    .set_text(&format!("{} TRACKS", album.tracks.len()));

                match album.album_art {
                    Some(filename) => {
                        let home = std::env::var("HOME").unwrap();
                        let path = format!("{}/.config/rockbox.org/covers/{}", home, filename);
                        self.album_cover.set_from_file(Some(&path));
                    }
                    None => {
                        self.album_cover
                            .set_resource(Some("/mg/tsirysndr/Rockbox/icons/jpg/albumart.jpg"));
                    }
                }

                let album_tracklist = self.album_tracklist.clone();
                while let Some(row) = album_tracklist.first_child() {
                    album_tracklist.remove(&row);
                }

                let discs = album
                    .tracks
                    .iter()
                    .map(|t| t.disc_number)
                    .max()
                    .unwrap_or(1);

                match discs > 1 {
                    true => {
                        for disc in 1..=discs {
                            let album_tracks = AlbumTracks::new();
                            let tracks = album
                                .tracks
                                .clone()
                                .into_iter()
                                .filter(|t| t.disc_number == disc)
                                .collect::<Vec<Track>>();
                            album_tracks
                                .imp()
                                .state
                                .set(Some(&self.state.upgrade().unwrap()));
                            album_tracks.imp().load_tracks(tracks, Some(disc));
                            self.album_tracklist.append(&album_tracks);
                        }
                    }
                    false => {
                        let album_tracks = AlbumTracks::new();
                        album_tracks
                            .imp()
                            .state
                            .set(Some(&self.state.upgrade().unwrap()));
                        album_tracks.imp().load_tracks(album.tracks, None);
                        self.album_tracklist.append(&album_tracks);
                    }
                }
            }
        }
    }
}

fn build_url() -> String {
    let host = env::var("ROCKBOX_HOST").unwrap_or_else(|_| "localhost".to_string());
    let port = env::var("ROCKBOX_PORT").unwrap_or_else(|_| "6061".to_string());

    format!("tcp://{}:{}", host, port)
}

glib::wrapper! {
  pub struct AlbumDetails(ObjectSubclass<imp::AlbumDetails>)
    @extends gtk::Widget, gtk::Box;
}

#[gtk::template_callbacks]
impl AlbumDetails {
    pub fn new() -> Self {
        glib::Object::new()
    }

    pub fn play_album(&self, shuffle: bool) {
        let album_id = self.imp().album_id.borrow();
        let album_id = album_id.clone();
        thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new().unwrap();
            let url = build_url();
            let _ = rt.block_on(async {
                let mut client = PlaybackServiceClient::connect(url).await?;
                client
                    .play_album(PlayAlbumRequest {
                        album_id,
                        shuffle: Some(shuffle),
                        ..Default::default()
                    })
                    .await?;
                Ok::<(), Error>(())
            });
        });
    }

    pub fn play_next(&self) {
        let album_id = self.imp().album_id.borrow();
        let album_id = album_id.clone();
        thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new().unwrap();
            let url = build_url();
            let _ = rt.block_on(async {
                let mut client = PlaylistServiceClient::connect(url).await?;
                client
                    .insert_album(InsertAlbumRequest {
                        album_id,
                        position: PLAYLIST_INSERT_FIRST,
                        ..Default::default()
                    })
                    .await?;
                Ok::<(), Error>(())
            });
        });
    }

    pub fn add_shuffled(&self) {
        let album_id = self.imp().album_id.borrow();
        let album_id = album_id.clone();
        thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new().unwrap();
            let url = build_url();
            let _ = rt.block_on(async {
                let mut client = PlaylistServiceClient::connect(url).await?;
                client
                    .insert_album(InsertAlbumRequest {
                        album_id,
                        position: PLAYLIST_INSERT_SHUFFLED,
                        ..Default::default()
                    })
                    .await?;
                Ok::<(), Error>(())
            });
        });
    }

    pub fn play_last_shuffled(&self) {
        let album_id = self.imp().album_id.borrow();
        let album_id = album_id.clone();
        thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new().unwrap();
            let url = build_url();
            let _ = rt.block_on(async {
                let mut client = PlaylistServiceClient::connect(url).await?;
                client
                    .insert_album(InsertAlbumRequest {
                        album_id,
                        position: PLAYLIST_INSERT_LAST_SHUFFLED,
                        ..Default::default()
                    })
                    .await?;
                Ok::<(), Error>(())
            });
        });
    }

    pub fn play_last(&self) {
        let album_id = self.imp().album_id.borrow();
        let album_id = album_id.clone();
        thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new().unwrap();
            let url = build_url();
            let _ = rt.block_on(async {
                let mut client = PlaylistServiceClient::connect(url).await?;
                client
                    .insert_album(InsertAlbumRequest {
                        album_id,
                        position: PLAYLIST_INSERT_LAST,
                        ..Default::default()
                    })
                    .await?;
                Ok::<(), Error>(())
            });
        });
    }
}
