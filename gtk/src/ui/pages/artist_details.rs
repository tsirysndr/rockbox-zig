use crate::api::rockbox::v1alpha1::library_service_client::LibraryServiceClient;
use crate::api::rockbox::v1alpha1::playback_service_client::PlaybackServiceClient;
use crate::api::rockbox::v1alpha1::playlist_service_client::PlaylistServiceClient;
use crate::api::rockbox::v1alpha1::{
    GetArtistRequest, GetArtistResponse, InsertArtistTracksRequest, PlayArtistTracksRequest,
};
use crate::constants::*;
use crate::state::AppState;
use crate::time::format_milliseconds;
use crate::ui::pages::album_details::AlbumDetails;
use crate::ui::song::Song;
use adw::prelude::*;
use adw::subclass::prelude::*;
use anyhow::Error;
use glib::subclass;
use gtk::glib;
use gtk::pango::EllipsizeMode;
use gtk::{CompositeTemplate, FlowBox, Image, Label, ListBox, Orientation};
use std::cell::RefCell;
use std::{env, thread};

mod imp {

    use super::*;

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(file = "../gtk/artist_details.ui")]
    pub struct ArtistDetails {
        #[template_child]
        pub artist_image: TemplateChild<Image>,
        #[template_child]
        pub artist_noimage: TemplateChild<gtk::Box>,
        #[template_child]
        pub artist_name: TemplateChild<Label>,
        #[template_child]
        pub tracks: TemplateChild<ListBox>,
        #[template_child]
        pub albums: TemplateChild<FlowBox>,

        pub main_stack: RefCell<Option<adw::ViewStack>>,
        pub album_details: RefCell<Option<AlbumDetails>>,
        pub library_page: RefCell<Option<adw::NavigationPage>>,
        pub state: glib::WeakRef<AppState>,
        pub artist_id: RefCell<String>,
        pub play_all_button: RefCell<Option<gtk::Button>>,
        pub shuffle_all_button: RefCell<Option<gtk::Button>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for ArtistDetails {
        const NAME: &'static str = "ArtistDetails";
        type ParentType = gtk::Box;
        type Type = super::ArtistDetails;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);

            klass.install_action(
                "app.artist.play",
                None,
                move |artist_details, _action, _target| {
                    artist_details.play(false);
                },
            );

            klass.install_action(
                "app.artist.shuffle",
                None,
                move |artist_details, _action, _target| {
                    artist_details.play(true);
                },
            );

            klass.install_action(
                "app.artist.play-next",
                None,
                move |artist_details, _action, _target| {
                    artist_details.play_next();
                },
            );

            klass.install_action(
                "app.artist.play-last",
                None,
                move |artist_details, _action, _target| {
                    artist_details.play_last();
                },
            );

            klass.install_action(
                "app.artist.add-shuffled",
                None,
                move |artist_details, _action, _target| {
                    artist_details.add_shuffled();
                },
            );

            klass.install_action(
                "app.artist.play-last-shuffled",
                None,
                move |artist_details, _action, _target| {
                    artist_details.play_last_shuffled();
                },
            );
        }

        fn instance_init(obj: &subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for ArtistDetails {
        fn constructed(&self) {
            self.parent_constructed();
        }
    }

    impl WidgetImpl for ArtistDetails {}
    impl BoxImpl for ArtistDetails {}

    impl ArtistDetails {
        pub fn set_main_stack(&self, main_stack: adw::ViewStack) {
            *self.main_stack.borrow_mut() = Some(main_stack);
        }

        pub fn set_album_details(&self, album_details: AlbumDetails) {
            *self.album_details.borrow_mut() = Some(album_details);
        }

        pub fn set_library_page(&self, library_page: adw::NavigationPage) {
            *self.library_page.borrow_mut() = Some(library_page);
        }

        pub fn hide_top_buttons(&self, hide: bool) {
            let play_all_button = self.play_all_button.borrow();
            let play_all_button_ref = play_all_button.as_ref();
            let shuffle_all_button = self.shuffle_all_button.borrow();
            let shuffle_all_button_ref = shuffle_all_button.as_ref();
            play_all_button_ref.unwrap().set_visible(!hide);
            shuffle_all_button_ref.unwrap().set_visible(!hide);
        }

        pub fn load_artist(&self, id: &str) {
            let id = id.to_string();
            let rt = tokio::runtime::Runtime::new().unwrap();
            self.artist_id.replace(id.clone());
            let response_ = rt.block_on(async {
                let url = build_url();
                let mut client = LibraryServiceClient::connect(url).await?;
                let response = client
                    .get_artist(GetArtistRequest { id })
                    .await?
                    .into_inner();
                Ok::<GetArtistResponse, Error>(response)
            });

            if let Ok(response) = response_ {
                if let Some(artist) = response.artist {
                    self.artist_name.set_text(&artist.name);
                    let tracks = self.tracks.clone();
                    while let Some(row) = tracks.first_child() {
                        tracks.remove(&row);
                    }

                    let state = self.state.upgrade().unwrap();

                    for track in artist.tracks.iter().take(10) {
                        let song = Song::new();
                        song.imp().state.set(Some(&state));
                        song.imp().track.replace(Some(track.clone()));
                        song.imp().track_title.set_text(&track.title);
                        song.imp().track_number.set_visible(false);
                        song.imp().artist.set_text(&track.artist);
                        song.imp()
                            .track_duration
                            .set_text(&format_milliseconds(track.length as u64));

                        match state.is_liked_track(&track.id) {
                            true => song.imp().heart_icon.set_icon_name(Some("heart-symbolic")),
                            false => song
                                .imp()
                                .heart_icon
                                .set_icon_name(Some("heart-outline-symbolic")),
                        }

                        match track.album_art.as_ref() {
                            Some(filename) => {
                                let home = env::var("HOME").unwrap();
                                let path =
                                    format!("{}/.config/rockbox.org/covers/{}", home, filename);
                                song.imp().album_art.set_from_file(Some(&path));
                            }
                            None => {
                                song.imp().album_art.set_resource(Some(
                                    "/mg/tsirysndr/Rockbox/icons/jpg/albumart.jpg",
                                ));
                            }
                        }

                        song.imp().album_art_container.set_margin_start(15);
                        song.imp().album_art_container.set_visible(true);
                        self.tracks.append(&song);
                    }

                    let albums = self.albums.clone();
                    while let Some(row) = albums.first_child() {
                        albums.remove(&row);
                    }

                    for album in artist.albums.iter().take(5) {
                        let image = match album.album_art.as_ref() {
                            Some(filename) => {
                                let home = env::var("HOME").unwrap();
                                let path =
                                    format!("{}/.config/rockbox.org/covers/{}", home, filename);
                                Image::from_file(&path)
                            }
                            None => {
                                Image::from_resource("/mg/tsirysndr/Rockbox/icons/jpg/albumart.jpg")
                            }
                        };
                        image.set_size_request(200, 200);
                        let image_container = gtk::Box::new(Orientation::Vertical, 0);
                        image_container.append(&image);

                        let gesture = gtk::GestureClick::new();
                        let album_id = album.id.clone();
                        let self_weak = self.downgrade();
                        gesture.connect_released(move |_, _, _, _| {
                            let self_ = match self_weak.upgrade() {
                                Some(self_) => self_,
                                None => return,
                            };
                            let obj = self_.obj();
                            obj.navigate_to_album(&album_id);
                            obj.imp()
                                .album_details
                                .borrow()
                                .as_ref()
                                .unwrap()
                                .imp()
                                .load_album(&album_id);
                        });

                        image_container.add_controller(gesture);

                        let title_click = gtk::GestureClick::new();
                        let album_id = album.id.clone();

                        let self_weak = self.downgrade();
                        title_click.connect_released(move |_, _, _, _| {
                            let self_ = match self_weak.upgrade() {
                                Some(self_) => self_,
                                None => return,
                            };
                            let obj = self_.obj();
                            obj.navigate_to_album(&album_id);
                            obj.imp()
                                .album_details
                                .borrow()
                                .as_ref()
                                .unwrap()
                                .imp()
                                .load_album(&album_id);
                        });

                        let title = Label::new(Some(&album.title));
                        title.set_ellipsize(EllipsizeMode::End);
                        title.set_max_width_chars(23);
                        title.add_css_class("album-label");
                        title.set_halign(gtk::Align::Start);
                        title.add_controller(title_click);

                        let artist = Label::new(Some(&album.artist));
                        artist.set_ellipsize(EllipsizeMode::End);
                        artist.set_max_width_chars(23);
                        artist.set_halign(gtk::Align::Start);

                        let year_u32 = album.year as u32;
                        let year = Label::new(Some(&album.year.to_string()));
                        year.set_halign(gtk::Align::Start);
                        year.add_css_class("year-label");

                        let infos = gtk::Box::new(Orientation::Vertical, 4);
                        infos.append(&title);
                        infos.append(&artist);

                        if year_u32 != 0 {
                            infos.append(&year);
                        }

                        infos.set_halign(gtk::Align::Start);

                        let container = gtk::Box::new(Orientation::Vertical, 10);
                        container.append(&image_container);
                        container.append(&infos);
                        container.set_margin_bottom(56);

                        self.albums.append(&container);
                    }
                }
            }
        }
    }
}

glib::wrapper! {
  pub struct ArtistDetails(ObjectSubclass<imp::ArtistDetails>)
    @extends gtk::Widget, gtk::Box;
}

#[gtk::template_callbacks]
impl ArtistDetails {
    pub fn new() -> Self {
        glib::Object::new()
    }

    pub fn navigate_to_album(&self, album_id: &str) {
        if let Some(main_stack) = self.imp().main_stack.borrow().as_ref() {
            main_stack.set_visible_child_name("album-details-page");

            if let Some(library_page) = self.imp().library_page.borrow().as_ref() {
                library_page.set_title("Album");
            }
            let state = self.imp().state.upgrade().unwrap();
            state.push_navigation("Album", "album-details-page");
        }
    }

    pub fn play(&self, shuffle: bool) {
        let artist_id = self.imp().artist_id.borrow();
        let artist_id = artist_id.clone();
        thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new().unwrap();
            let url = build_url();
            let _ = rt.block_on(async {
                let mut client = PlaybackServiceClient::connect(url).await?;
                client
                    .play_artist_tracks(PlayArtistTracksRequest {
                        artist_id,
                        shuffle: Some(shuffle),
                        ..Default::default()
                    })
                    .await?;
                Ok::<(), Error>(())
            });
        });
    }

    pub fn play_next(&self) {
        let artist_id = self.imp().artist_id.borrow();
        let artist_id = artist_id.clone();
        thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new().unwrap();
            let url = build_url();
            let _ = rt.block_on(async {
                let mut client = PlaylistServiceClient::connect(url).await?;
                client
                    .insert_artist_tracks(InsertArtistTracksRequest {
                        artist_id,
                        position: PLAYLIST_INSERT_FIRST,
                        ..Default::default()
                    })
                    .await?;
                Ok::<(), Error>(())
            });
        });
    }

    pub fn play_last(&self) {
        let artist_id = self.imp().artist_id.borrow();
        let artist_id = artist_id.clone();
        thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new().unwrap();
            let url = build_url();
            let _ = rt.block_on(async {
                let mut client = PlaylistServiceClient::connect(url).await?;
                client
                    .insert_artist_tracks(InsertArtistTracksRequest {
                        artist_id,
                        position: PLAYLIST_INSERT_LAST,
                        ..Default::default()
                    })
                    .await?;
                Ok::<(), Error>(())
            });
        });
    }

    pub fn add_shuffled(&self) {
        let artist_id = self.imp().artist_id.borrow();
        let artist_id = artist_id.clone();
        thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new().unwrap();
            let url = build_url();
            let _ = rt.block_on(async {
                let mut client = PlaylistServiceClient::connect(url).await?;
                client
                    .insert_artist_tracks(InsertArtistTracksRequest {
                        artist_id,
                        position: PLAYLIST_INSERT_SHUFFLED,
                        ..Default::default()
                    })
                    .await?;
                Ok::<(), Error>(())
            });
        });
    }

    pub fn play_last_shuffled(&self) {
        let artist_id = self.imp().artist_id.borrow();
        let artist_id = artist_id.clone();
        thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new().unwrap();
            let url = build_url();
            let _ = rt.block_on(async {
                let mut client = PlaylistServiceClient::connect(url).await?;
                client
                    .insert_artist_tracks(InsertArtistTracksRequest {
                        artist_id,
                        position: PLAYLIST_INSERT_LAST_SHUFFLED,
                        ..Default::default()
                    })
                    .await?;
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
