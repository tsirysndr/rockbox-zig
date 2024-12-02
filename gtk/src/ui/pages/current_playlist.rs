use crate::api::rockbox::v1alpha1::library_service_client::LibraryServiceClient;
use crate::api::rockbox::v1alpha1::playback_service_client::PlaybackServiceClient;
use crate::api::rockbox::v1alpha1::playlist_service_client::PlaylistServiceClient;
use crate::api::rockbox::v1alpha1::{
    CurrentTrackResponse, GetCurrentRequest, GetCurrentResponse, GetTracksRequest,
    GetTracksResponse, PlaylistResponse, StreamPlaylistRequest,
};
use crate::state::AppState;
use crate::time::format_milliseconds;
use crate::ui::song::Song;
use adw::prelude::*;
use adw::subclass::prelude::*;
use anyhow::Error;
use glib::subclass;
use gtk::glib;
use gtk::{CompositeTemplate, Image, Label, ListBox};
use std::cell::{Cell, RefCell};
use std::env;
use std::thread;
use tokio::sync::mpsc;

mod imp {

    use super::*;

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(file = "../gtk/current_playlist.ui")]
    pub struct CurrentPlaylist {
        #[template_child]
        pub album_cover: TemplateChild<Image>,
        #[template_child]
        pub track_title: TemplateChild<Label>,
        #[template_child]
        pub track_artist: TemplateChild<Label>,
        #[template_child]
        pub now_playing: TemplateChild<gtk::Box>,
        #[template_child]
        pub next_tracks: TemplateChild<ListBox>,

        pub state: glib::WeakRef<AppState>,
        pub ready: Cell<bool>,
        pub tracks: RefCell<Vec<CurrentTrackResponse>>,
        pub current_index: Cell<usize>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for CurrentPlaylist {
        const NAME: &'static str = "CurrentPlaylist";
        type ParentType = gtk::Box;
        type Type = super::CurrentPlaylist;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
        }

        fn instance_init(obj: &subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for CurrentPlaylist {
        fn constructed(&self) {
            self.parent_constructed();

            let self_weak = self.downgrade();
            glib::idle_add_local(move || {
                let (tx, mut rx) = mpsc::channel(32);

                let self_ = match self_weak.upgrade() {
                    Some(self_) => self_,
                    None => return glib::ControlFlow::Continue,
                };

                glib::MainContext::default().spawn_local(async move {
                    let obj = self_.obj();
                    obj.stream_playlist(tx);
                });

                let self_ = match self_weak.upgrade() {
                    Some(self_) => self_,
                    None => return glib::ControlFlow::Continue,
                };

                glib::MainContext::default().spawn_local(async move {
                    let obj = self_.obj();
                    while let Some(playlist) = rx.recv().await {
                        obj.load_current_track();
                        let state = obj.imp().state.upgrade().unwrap();

                        obj.imp().current_index.set(playlist.index as usize);
                        obj.imp().tracks.replace(playlist.tracks.clone());

                        if let Some(track) = state.current_track() {
                            if track.elapsed > 1000 && obj.imp().ready.get() {
                                continue;
                            }
                        }

                        obj.imp().ready.set(true);

                        let index = playlist.index as usize + 1;
                        let mut tracks = playlist.tracks.clone();
                        let next_tracks =
                            tracks.drain(index..).collect::<Vec<CurrentTrackResponse>>();

                        while let Some(row) = obj.imp().now_playing.first_child() {
                            obj.imp().now_playing.remove(&row);
                        }

                        let label = Label::new(Some("Now playing"));
                        label.set_halign(gtk::Align::Start);
                        label.set_margin_start(10);
                        label.add_css_class("bold");

                        obj.imp().now_playing.append(&label);

                        if let Some(track) = state.current_track() {
                            let song = Song::new();
                            song.imp().track_number.set_visible(false);
                            song.imp().track_title.set_text(&track.title);
                            song.imp().artist.set_text(&track.artist);
                            song.imp().track_duration.set_visible(false);
                            song.imp().heart_button.set_visible(false);
                            song.imp().more_button.set_visible(false);

                            match track.album_art {
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

                            song.imp().album_art_container.set_visible(true);
                            obj.imp().now_playing.append(&song);
                        }

                        while let Some(row) = obj.imp().next_tracks.first_child() {
                            obj.imp().next_tracks.remove(&row);
                        }

                        for track in next_tracks.into_iter().take(20) {
                            let song = Song::new();
                            song.imp().track_number.set_visible(false);
                            song.imp().track_title.set_text(&track.title);
                            song.imp().artist.set_text(&track.artist);
                            song.imp().track_duration.set_visible(false);
                            song.imp().heart_button.set_visible(false);
                            song.imp().more_button.set_visible(false);

                            match track.album_art {
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

                            song.imp().album_art_container.set_visible(true);
                            obj.imp().next_tracks.append(&song);
                        }
                    }
                });

                glib::ControlFlow::Break
            });
        }
    }

    impl WidgetImpl for CurrentPlaylist {}
    impl BoxImpl for CurrentPlaylist {}
}

glib::wrapper! {
  pub struct CurrentPlaylist(ObjectSubclass<imp::CurrentPlaylist>)
    @extends gtk::Widget, gtk::Box;
}

#[gtk::template_callbacks]
impl CurrentPlaylist {
    pub fn new() -> Self {
        glib::Object::new()
    }

    pub fn load_current_track(&self) {
        let state = self.imp().state.upgrade().unwrap();
        match state.current_track() {
            Some(track) => {
                self.imp().track_title.set_text(&track.title);
                self.imp().track_artist.set_text(&track.artist);
                match track.album_art {
                    Some(filename) => {
                        let home = env::var("HOME").unwrap();
                        let path = format!("{}/.config/rockbox.org/covers/{}", home, filename);
                        self.imp().album_cover.set_from_file(Some(&path));
                    }
                    None => {
                        self.imp()
                            .album_cover
                            .set_resource(Some("/mg/tsirysndr/Rockbox/icons/jpg/albumart.jpg"));
                    }
                }
            }
            None => {
                self.imp()
                    .album_cover
                    .set_resource(Some("/mg/tsirysndr/Rockbox/icons/jpg/albumart.jpg"));
                self.imp().track_title.set_text("No song playing");
                self.imp().track_artist.set_text("");
            }
        }
    }

    pub fn load_current_playlist(&self) {
        let index = self.imp().current_index.get() + 1;
        let mut tracks = self.imp().tracks.borrow().clone();
        let next_tracks = tracks.drain(index..).collect::<Vec<CurrentTrackResponse>>();

        while let Some(row) = self.imp().now_playing.first_child() {
            self.imp().now_playing.remove(&row);
        }

        let label = Label::new(Some("Now playing"));
        label.set_halign(gtk::Align::Start);
        label.set_margin_start(10);
        label.add_css_class("bold");

        self.imp().now_playing.append(&label);
        let state = self.imp().state.upgrade().unwrap();

        if let Some(track) = state.current_track() {
            let song = Song::new();
            song.imp().track_number.set_visible(false);
            song.imp().track_title.set_text(&track.title);
            song.imp().artist.set_text(&track.artist);
            song.imp().track_duration.set_visible(false);
            song.imp().heart_button.set_visible(false);
            song.imp().more_button.set_visible(false);

            match track.album_art {
                Some(filename) => {
                    let home = env::var("HOME").unwrap();
                    let path = format!("{}/.config/rockbox.org/covers/{}", home, filename);
                    song.imp().album_art.set_from_file(Some(&path));
                }
                None => {
                    song.imp()
                        .album_art
                        .set_resource(Some("/mg/tsirysndr/Rockbox/icons/jpg/albumart.jpg"));
                }
            }

            song.imp().album_art_container.set_visible(true);
            self.imp().now_playing.append(&song);
        }

        while let Some(row) = self.imp().next_tracks.first_child() {
            self.imp().next_tracks.remove(&row);
        }

        for track in next_tracks.into_iter().take(20) {
            let song = Song::new();
            song.imp().track_number.set_visible(false);
            song.imp().track_title.set_text(&track.title);
            song.imp().artist.set_text(&track.artist);
            song.imp().track_duration.set_visible(false);
            song.imp().heart_button.set_visible(false);
            song.imp().more_button.set_visible(false);

            match track.album_art {
                Some(filename) => {
                    let home = env::var("HOME").unwrap();
                    let path = format!("{}/.config/rockbox.org/covers/{}", home, filename);
                    song.imp().album_art.set_from_file(Some(&path));
                }
                None => {
                    song.imp()
                        .album_art
                        .set_resource(Some("/mg/tsirysndr/Rockbox/icons/jpg/albumart.jpg"));
                }
            }

            song.imp().album_art_container.set_visible(true);
            self.imp().next_tracks.append(&song);
        }
    }

    pub fn stream_playlist(&self, tx: mpsc::Sender<PlaylistResponse>) {
        thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new().unwrap();
            let _ = rt.block_on(async {
                let url = build_url();
                let mut client = PlaybackServiceClient::connect(url).await?;
                let mut stream = client
                    .stream_playlist(StreamPlaylistRequest {})
                    .await?
                    .into_inner();

                while let Some(playlist) = stream.message().await? {
                    tx.send(playlist).await?;
                }

                Ok::<(), Error>(())
            });
        });
    }
}

fn build_url() -> String {
    let host = env::var("ROCKBOX_HOST").unwrap_or_else(|_| "localhost".to_string());
    let port = env::var("ROCKBOX_PORT").unwrap_or_else(|_| "6061".to_string());
    format!("tcp://{}:{}", host, port)
}
