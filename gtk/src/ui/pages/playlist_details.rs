use crate::api::rockbox::v1alpha1::playlist_service_client::PlaylistServiceClient;
use crate::api::rockbox::v1alpha1::{GetPlaylistRequest, GetPlaylistResponse, Track};
use crate::state::AppState;
use crate::ui::pages::likes::Likes;
use crate::{time::format_milliseconds, ui::song::Song};
use adw::prelude::*;
use adw::subclass::prelude::*;
use anyhow::Error;
use glib::subclass;
use gtk::glib;
use gtk::pango::EllipsizeMode;
use gtk::{Button, CompositeTemplate, ListBox};
use std::cell::Cell;
use std::cell::RefCell;
use std::env;

mod imp {

    use super::*;

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(resource = "/io/github/tsirysndr/Rockbox/gtk/playlist_details.ui")]
    pub struct PlaylistDetails {
        #[template_child]
        pub tracks: TemplateChild<ListBox>,

        pub main_stack: RefCell<Option<adw::ViewStack>>,
        pub go_back_button: RefCell<Option<Button>>,
        pub state: glib::WeakRef<AppState>,
        pub all_tracks: RefCell<Vec<Track>>,
        pub size: Cell<usize>,
        pub state: glib::WeakRef<AppState>,
        pub likes_page: RefCell<Option<Likes>>,
        pub search_mode: Cell<bool>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for PlaylistDetails {
        const NAME: &'static str = "PlaylistDetails";
        type ParentType = gtk::Box;
        type Type = super::PlaylistDetails;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
        }

        fn instance_init(obj: &subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for PlaylistDetails {
        fn constructed(&self) {
            self.parent_constructed();
            self.size.set(20);
        }
    }

    impl WidgetImpl for PlaylistDetails {}
    impl BoxImpl for PlaylistDetails {}

    impl PlaylistDetails {
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
  pub struct PlaylistDetails(ObjectSubclass<imp::PlaylistDetails>)
    @extends gtk::Widget, gtk::Box;
}

#[gtk::template_callbacks]
impl PlaylistDetails {
    pub fn new() -> Self {
        glib::Object::new()
    }

    pub fn load_tracks(&self, playlist_id: String) {
        let rt = tokio::runtime::Runtime::new().unwrap();
        let response_ = rt.block_on(async {
            let url = build_url();
            let mut client = PlaylistServiceClient::connect(url).await?;
            let response = client
                .get_playlist(GetPlaylistRequest { playlist_id })
                .await?
                .into_inner();
            Ok::<GetPlaylistResponse, Error>(response)
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
}

fn build_url() -> String {
    let host = env::var("ROCKBOX_HOST").unwrap_or_else(|_| "localhost".to_string());
    let port = env::var("ROCKBOX_PORT").unwrap_or_else(|_| "6061".to_string());
    format!("tcp://{}:{}", host, port)
}
