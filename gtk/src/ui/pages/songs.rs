use crate::api::rockbox::v1alpha1::library_service_client::LibraryServiceClient;
use crate::api::rockbox::v1alpha1::{
    GetArtistRequest, GetArtistResponse, GetTracksRequest, GetTracksResponse, SearchRequest, Track,
};
use crate::state::AppState;
use crate::time::format_milliseconds;
use crate::ui::pages::likes::Likes;
use crate::ui::song::Song;
use adw::prelude::*;
use adw::subclass::prelude::*;
use anyhow::Error;
use glib::subclass;
use gtk::glib;
use gtk::pango::EllipsizeMode;
use gtk::{CompositeTemplate, ListBox};
use std::cell::{Cell, RefCell};
use std::{env, thread};

mod imp {
    use super::*;

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(resource = "/io/github/tsirysndr/Rockbox/gtk/songs.ui")]
    pub struct Songs {
        #[template_child]
        pub tracks: TemplateChild<ListBox>,

        pub all_tracks: RefCell<Vec<Track>>,
        pub size: Cell<usize>,
        pub state: glib::WeakRef<AppState>,
        pub likes_page: RefCell<Option<Likes>>,
        pub search_mode: Cell<bool>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for Songs {
        const NAME: &'static str = "Songs";
        type ParentType = gtk::Box;
        type Type = super::Songs;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
        }

        fn instance_init(obj: &subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for Songs {
        fn constructed(&self) {
            self.parent_constructed();
            self.size.set(20);

            let self_weak = self.downgrade();
            glib::idle_add_local(move || {
                let self_ = match self_weak.upgrade() {
                    Some(self_) => self_,
                    None => return glib::ControlFlow::Continue,
                };

                glib::spawn_future_local(async move {
                    let obj = self_.obj();
                    let likes_page = obj.imp().likes_page.borrow();
                    let likes_page_ref = likes_page.as_ref().unwrap();
                    likes_page_ref.load_likes();

                    if obj.imp().search_mode.get() {
                        return;
                    }

                    obj.load_songs();
                });
                glib::ControlFlow::Break
            });
        }
    }

    impl WidgetImpl for Songs {}
    impl BoxImpl for Songs {}

    impl Songs {
        pub fn create_songs_widgets(&self, list: Option<Vec<Track>>, limit: Option<usize>) {
            let tracks = self.all_tracks.borrow();
            let tracks = match list {
                Some(list) => {
                    let cloned_list = list.clone();
                    cloned_list.into_iter().enumerate().take(list.len())
                }
                None => {
                    let cloned_tracks = tracks.clone();
                    cloned_tracks
                        .into_iter()
                        .enumerate()
                        .take(limit.unwrap_or(tracks.len()))
                }
            };

            let size = self.size.get();
            let state = self.state.upgrade().unwrap();

            for (index, track) in tracks {
                let song = Song::new();
                song.imp().state.set(Some(&state));
                song.imp().track.replace(Some(track.clone()));
                song.imp().track_number.set_text(&format!(
                    "{}",
                    match size == 20 {
                        true => index + 1,
                        false => index + 1 + size - 3,
                    }
                ));
                song.imp().track_title.set_text(&track.title);
                song.imp().track_title.set_ellipsize(EllipsizeMode::End);
                song.imp().track_title.set_max_width_chars(100);
                song.imp().artist.set_text(&track.artist);
                song.imp().artist.set_ellipsize(EllipsizeMode::End);
                song.imp().artist.set_max_width_chars(100);
                song.imp()
                    .track_duration
                    .set_text(&format!("{}", format_milliseconds(track.length as u64)));

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
                        let path = format!("{}/.config/rockbox.org/covers/{}", home, filename);
                        song.imp().album_art.set_from_file(Some(&path));
                    }
                    None => {
                        song.imp().album_art.set_resource(Some(
                            "/io/github/tsirysndr/Rockbox/icons/jpg/albumart.jpg",
                        ));
                    }
                }

                song.imp().album_art_container.set_visible(true);
                self.tracks.append(&song);
            }
        }
    }
}

glib::wrapper! {
  pub struct Songs(ObjectSubclass<imp::Songs>)
    @extends gtk::Widget, gtk::Box;
}

#[gtk::template_callbacks]
impl Songs {
    pub fn new() -> Self {
        glib::Object::new()
    }

    pub fn load_songs(&self) {
        let rt = tokio::runtime::Runtime::new().unwrap();
        let response_ = rt.block_on(async {
            let url = build_url();
            let mut client = LibraryServiceClient::connect(url).await?;
            let response = client.get_tracks(GetTracksRequest {}).await?.into_inner();
            Ok::<GetTracksResponse, Error>(response)
        });

        if let Ok(response) = response_ {
            let state = self.imp().state.upgrade().unwrap();
            state.set_tracks(response.tracks.clone());

            let tracks = self.imp().tracks.clone();
            while let Some(row) = tracks.first_child() {
                tracks.remove(&row);
            }

            self.imp().all_tracks.replace(response.tracks.clone());
            self.imp().create_songs_widgets(None, Some(20));
        }
    }

    pub fn load_artist_songs(&self, artist_id: &str) {
        self.imp().size.set(20);
        let rt = tokio::runtime::Runtime::new().unwrap();
        let response_ = rt.block_on(async {
            let url = build_url();
            let mut client = LibraryServiceClient::connect(url).await?;
            let response = client
                .get_artist(GetArtistRequest {
                    id: artist_id.to_string(),
                })
                .await?
                .into_inner();
            Ok::<GetArtistResponse, Error>(response)
        });

        if let Ok(response) = response_ {
            let tracks = self.imp().tracks.clone();
            while let Some(row) = tracks.first_child() {
                tracks.remove(&row);
            }

            if let Some(artist) = response.artist {
                self.imp().all_tracks.replace(artist.tracks.clone());
                self.imp().create_songs_widgets(None, Some(20));
            }
        }
    }

    pub fn clear(&self, ui_only: bool) {
        if !ui_only {
            let state = self.imp().state.upgrade().unwrap();
            state.clear_search_results();
        }
        let tracks_ = self.imp().tracks.clone();
        while let Some(row) = tracks_.first_child() {
            tracks_.remove(&row);
        }
    }

    pub fn load_search_results(&self, tracks: Vec<Track>) {
        self.clear(true);

        self.imp().all_tracks.replace(tracks.clone());
        self.imp().create_songs_widgets(Some(tracks.clone()), None);
    }
}

fn build_url() -> String {
    let host = env::var("ROCKBOX_HOST").unwrap_or_else(|_| "localhost".to_string());
    let port = env::var("ROCKBOX_PORT").unwrap_or_else(|_| "6061".to_string());
    format!("tcp://{}:{}", host, port)
}
