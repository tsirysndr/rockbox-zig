use std::{env, thread};

use crate::api::rockbox::v1alpha1::library_service_client::LibraryServiceClient;
use crate::api::rockbox::v1alpha1::playback_service_client::PlaybackServiceClient;
use crate::api::rockbox::v1alpha1::settings_service_client::SettingsServiceClient;
use crate::api::rockbox::v1alpha1::{
    CurrentTrackRequest, CurrentTrackResponse, GetGlobalSettingsRequest, GetGlobalSettingsResponse,
    LikeTrackRequest, NextRequest, PauseRequest, PlayRequest, PreviousRequest, ResumeRequest,
    SaveSettingsRequest, StreamCurrentTrackRequest, StreamStatusRequest, UnlikeTrackRequest,
};
use crate::state::AppState;
use crate::time::format_milliseconds;
use crate::types::track::Track;
use crate::ui::pages::album_details::AlbumDetails;
use crate::ui::pages::artist_details::ArtistDetails;
use crate::ui::pages::current_playlist::CurrentPlaylist;
use adw::prelude::*;
use adw::subclass::prelude::*;
use anyhow::Error;
use glib::subclass;
use gtk::glib;
use gtk::pango::EllipsizeMode;
use gtk::{Button, CompositeTemplate, Image, Label, MenuButton, Scale};
use std::cell::{Cell, RefCell};
use tokio::sync::mpsc;

mod imp {

    use super::*;

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(file = "gtk/media_controls.ui")]
    pub struct MediaControls {
        #[template_child]
        pub shuffle_button: TemplateChild<Button>,
        #[template_child]
        pub previous_button: TemplateChild<Button>,
        #[template_child]
        pub play_pause_button: TemplateChild<Button>,
        #[template_child]
        pub play_icon: TemplateChild<Image>,
        #[template_child]
        pub next_button: TemplateChild<Button>,
        #[template_child]
        pub repeat_button: TemplateChild<Button>,
        #[template_child]
        pub album_art: TemplateChild<Image>,
        #[template_child]
        pub current_song_details: TemplateChild<gtk::Box>,
        #[template_child]
        pub title: TemplateChild<Label>,
        #[template_child]
        pub artist_album: TemplateChild<Label>,
        #[template_child]
        pub elapsed: TemplateChild<Label>,
        #[template_child]
        pub duration: TemplateChild<Label>,
        #[template_child]
        pub media_control_bar_progress: TemplateChild<gtk::Box>,
        #[template_child]
        pub progress_bar: TemplateChild<Scale>,
        #[template_child]
        pub heart_button: TemplateChild<Button>,
        #[template_child]
        pub more_button: TemplateChild<MenuButton>,
        #[template_child]
        pub heart_icon: TemplateChild<Image>,
        #[template_child]
        pub playlist_button: TemplateChild<Button>,

        pub current_track: RefCell<Option<Track>>,
        pub playback_status: Cell<i32>,
        pub shuffle_enabled: Cell<bool>,
        pub repeat_mode: Cell<i32>,
        pub library_page: RefCell<Option<adw::NavigationPage>>,
        pub current_album_id: RefCell<Option<String>>,
        pub current_artist_id: RefCell<Option<String>>,
        pub album_details: RefCell<Option<AlbumDetails>>,
        pub artist_details: RefCell<Option<ArtistDetails>>,
        pub main_stack: RefCell<Option<adw::ViewStack>>,
        pub go_back_button: RefCell<Option<Button>>,
        pub playlist_displayed: Cell<bool>,
        pub state: glib::WeakRef<AppState>,
        pub current_playlist: RefCell<Option<CurrentPlaylist>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for MediaControls {
        const NAME: &'static str = "MediaControls";
        type ParentType = gtk::Box;
        type Type = super::MediaControls;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);

            klass.install_action(
                "app.play_pause",
                None,
                move |media_controls, _action, _target| {
                    media_controls.play();
                },
            );

            klass.install_action(
                "app.previous",
                None,
                move |media_controls, _action, _target| {
                    media_controls.previous();
                },
            );

            klass.install_action("app.next", None, move |media_controls, _action, _target| {
                media_controls.next();
            });

            klass.install_action(
                "app.shuffle",
                None,
                move |media_controls, _action, _target| {
                    media_controls.shuffle();
                },
            );

            klass.install_action(
                "app.repeat",
                None,
                move |media_controls, _action, _target| {
                    media_controls.repeat();
                },
            );

            klass.install_action("app.like", None, move |media_controls, _action, _target| {
                media_controls.like();
            });

            klass.install_action(
                "app.show-playlist",
                None,
                move |media_controls, _action, _target| {
                    media_controls.show_playlist();
                },
            );

            klass.install_action(
                "app.go-to-artist",
                None,
                move |media_controls, _action, _target| {
                    media_controls.go_to_artist();
                },
            );

            klass.install_action(
                "app.go-to-album",
                None,
                move |media_controls, _action, _target| {
                    media_controls.go_to_album();
                },
            );
        }

        fn instance_init(obj: &subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for MediaControls {
        fn constructed(&self) {
            self.parent_constructed();

            let self_weak = self.downgrade();
            self.progress_bar
                .connect_change_value(move |_, _scroll_type, value| {
                    let self_ = match self_weak.upgrade() {
                        Some(self_) => self_,
                        None => return glib::Propagation::Stop,
                    };
                    let current_track = self_.current_track.borrow();
                    if let Some(track) = &*current_track {
                        let elapsed = (track.duration as i64 * value as i64) / 100;
                        glib::MainContext::default().spawn_local(async move {
                            let rt = tokio::runtime::Runtime::new().unwrap();
                            let _ = rt.block_on(async {
                                let url = build_url();
                                let mut client = PlaybackServiceClient::connect(url).await?;
                                client.play(PlayRequest { elapsed, offset: 0 }).await?;
                                Ok::<(), Error>(())
                            });
                        });
                    }
                    glib::Propagation::Stop
                });

            let album_click = gtk::GestureClick::new();
            let self_weak = self.downgrade();
            album_click.connect_released(move |_, _, _, _| {
                let self_ = match self_weak.upgrade() {
                    Some(self_) => self_,
                    None => return,
                };
                let state = self_.state.upgrade().unwrap();
                let current_album_id = self_.current_album_id.borrow();
                if let Some(album_id) = current_album_id.as_ref() {
                    let album_details = self_.album_details.borrow();
                    let library_page = self_.library_page.borrow();
                    let main_stack = self_.main_stack.borrow();
                    let go_back_button = self_.go_back_button.borrow();
                    let album_details_ref = album_details.as_ref().unwrap();
                    let library_page_ref = library_page.as_ref().unwrap();
                    let main_stack_ref = main_stack.as_ref().unwrap();
                    let go_back_button_ref = go_back_button.as_ref().unwrap();
                    main_stack_ref.set_visible_child_name("album-details-page");
                    library_page_ref.set_title("Album");
                    go_back_button_ref.set_visible(true);
                    album_details_ref.imp().hide_top_buttons(true);
                    state.push_navigation("Album", "album-details-page");
                    album_details_ref.imp().load_album(album_id);
                }
            });
            let album_art = self.album_art.get();
            album_art.add_controller(album_click);

            let self_weak = self.downgrade();
            glib::idle_add_local(move || {
                let self_ = match self_weak.upgrade() {
                    Some(self_) => self_,
                    None => return glib::ControlFlow::Continue,
                };

                let (tx, mut rx) = mpsc::channel(32);

                glib::MainContext::default().spawn_local(async move {
                    let obj = self_.obj();
                    obj.load_current_track(tx);
                });

                let self_ = match self_weak.upgrade() {
                    Some(self_) => self_,
                    None => return glib::ControlFlow::Continue,
                };

                glib::MainContext::default().spawn_local(async move {
                    let obj = self_.obj();
                    obj.load_playback_settings();
                });

                let self_ = match self_weak.upgrade() {
                    Some(self_) => self_,
                    None => return glib::ControlFlow::Continue,
                };

                let state = self_.state.upgrade().unwrap();

                glib::MainContext::default().spawn_local(async move {
                    while let Some(track) = rx.recv().await {
                        let mut current_artist_id = self_.current_artist_id.borrow_mut();
                        let title = self_.title.get();
                        let artist_album = self_.artist_album.get();
                        let elapsed = self_.elapsed.get();
                        let duration = self_.duration.get();
                        let album_art = self_.album_art.get();
                        let media_control_bar_progress = self_.media_control_bar_progress.get();
                        let progress_bar = self_.progress_bar.get();
                        let heart_button = self_.heart_button.get();
                        let more_button = self_.more_button.get();

                        if track.length == 0 {
                            media_control_bar_progress.set_visible(false);
                            heart_button.set_visible(false);
                            more_button.set_visible(false);
                            continue;
                        }

                        current_artist_id.replace(track.artist_id.clone());
                        let progression = (track.elapsed as f64 / track.length as f64) * 100.0;
                        progress_bar.set_value(progression);
                        media_control_bar_progress.set_visible(true);

                        heart_button.set_visible(true);
                        more_button.set_visible(true);

                        title.set_text(&track.title);
                        title.set_ellipsize(EllipsizeMode::End);
                        title.set_max_width_chars(100);
                        artist_album.set_text(&format!("{} - {}", track.artist, track.album));
                        artist_album.set_ellipsize(EllipsizeMode::End);
                        artist_album.set_max_width_chars(100);
                        elapsed.set_text(&format_milliseconds(track.elapsed));
                        duration.set_text(&format_milliseconds(track.length));

                        self_.current_album_id.replace(Some(track.album_id));

                        if let Some(ref filename) = track.album_art {
                            let home = std::env::var("HOME").unwrap();
                            let path = format!("{}/.config/rockbox.org/covers/{}", home, filename);
                            album_art.set_from_file(Some(&path));
                        } else {
                            album_art
                                .set_resource(Some("/mg/tsirysndr/Rockbox/icons/jpg/albumart.jpg"));
                        }

                        match state.is_liked_track(&track.id) {
                            true => self_.heart_icon.set_icon_name(Some("heart-symbolic")),
                            false => self_
                                .heart_icon
                                .set_icon_name(Some("heart-outline-symbolic")),
                        }

                        let current_track = Track {
                            id: track.id,
                            title: track.title,
                            artist: track.artist,
                            album: track.album,
                            album_artist: track.album_artist,
                            duration: track.length,
                            elapsed: track.elapsed,
                            album_art: track.album_art,
                            ..Default::default()
                        };

                        state.set_current_track(current_track.clone());
                        self_.set_current_track(current_track.clone());
                    }
                });

                let (tx, mut rx) = mpsc::channel(32);

                let self_ = match self_weak.upgrade() {
                    Some(self_) => self_,
                    None => return glib::ControlFlow::Continue,
                };

                glib::MainContext::default().spawn_local(async move {
                    let obj = self_.obj();
                    obj.stream_status(tx);
                });

                let self_ = match self_weak.upgrade() {
                    Some(self_) => self_,
                    None => return glib::ControlFlow::Continue,
                };

                glib::MainContext::default().spawn_local(async move {
                    while let Some(status) = rx.recv().await {
                        self_.playback_status.set(status);
                        match status {
                            1 => self_.play_icon.set_icon_name(Some("media-playback-pause")),
                            3 => self_.play_icon.set_icon_name(Some("media-playback-start")),
                            _ => {}
                        }
                    }
                });

                glib::ControlFlow::Break
            });
        }
    }

    impl WidgetImpl for MediaControls {}
    impl BoxImpl for MediaControls {}

    impl MediaControls {
        pub fn set_current_track(&self, track: Track) {
            let mut current_track = self.current_track.borrow_mut();
            *current_track = Some(track);
        }
    }
}

glib::wrapper! {
  pub struct MediaControls(ObjectSubclass<imp::MediaControls>)
    @extends gtk::Widget, gtk::Box;
}

fn build_url() -> String {
    let host = env::var("ROCKBOX_HOST").unwrap_or_else(|_| "localhost".to_string());
    let port = env::var("ROCKBOX_PORT").unwrap_or_else(|_| "6061".to_string());

    format!("tcp://{}:{}", host, port)
}

#[gtk::template_callbacks]
impl MediaControls {
    pub fn new() -> Self {
        glib::Object::new()
    }

    pub fn load_playback_settings(&self) {
        let rt = tokio::runtime::Runtime::new().unwrap();
        if let Ok(settings) = rt.block_on(async {
            let url = build_url();
            let mut client = SettingsServiceClient::connect(url).await?;
            let response = client
                .get_global_settings(GetGlobalSettingsRequest {})
                .await?
                .into_inner();

            Ok::<GetGlobalSettingsResponse, Error>(response)
        }) {
            self.imp().shuffle_enabled.set(settings.playlist_shuffle);
            self.imp().repeat_mode.set(settings.repeat_mode);

            match self.imp().shuffle_enabled.get() {
                true => self
                    .imp()
                    .shuffle_button
                    .remove_css_class("inactive-button"),
                false => self.imp().shuffle_button.add_css_class("inactive-button"),
            }

            match self.imp().repeat_mode.get() {
                0 => self.imp().repeat_button.add_css_class("inactive-button"),
                _ => self.imp().repeat_button.remove_css_class("inactive-button"),
            }

            return;
        }
        println!("playback: failed to load settings");
    }

    pub fn stream_status(&self, tx: mpsc::Sender<i32>) {
        thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new().unwrap();
            let _ = rt.block_on(async {
                let url = build_url();
                let mut client = PlaybackServiceClient::connect(url).await?;
                let mut stream = client
                    .stream_status(StreamStatusRequest {})
                    .await?
                    .into_inner();

                while let Some(status) = stream.message().await? {
                    tx.send(status.status).await?;
                }

                Ok::<(), Error>(())
            });
        });
    }

    pub fn load_current_track(&self, tx: mpsc::Sender<CurrentTrackResponse>) {
        thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new().unwrap();
            let _ = rt.block_on(async {
                let url = build_url();
                let mut client = PlaybackServiceClient::connect(url).await?;
                let mut stream = client
                    .stream_current_track(StreamCurrentTrackRequest {})
                    .await?
                    .into_inner();

                while let Some(track) = stream.message().await? {
                    tx.send(track).await?;
                }

                Ok::<(), Error>(())
            });
        });

        let handle = thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                let url = build_url();
                let mut client = PlaybackServiceClient::connect(url).await?;
                let response = client.current_track(CurrentTrackRequest {}).await?;
                Ok::<CurrentTrackResponse, Error>(response.into_inner())
            })
        });

        if let Ok(track) = handle.join().unwrap() {
            let title = self.imp().title.get();
            let artist_album = self.imp().artist_album.get();
            let elapsed = self.imp().elapsed.get();
            let duration = self.imp().duration.get();
            let album_art = self.imp().album_art.get();
            let media_control_bar_progress = self.imp().media_control_bar_progress.get();
            let progress_bar = self.imp().progress_bar.get();

            if track.length == 0 {
                return;
            }

            let progression = (track.elapsed as f64 / track.length as f64) * 100.0;
            progress_bar.set_value(progression);
            media_control_bar_progress.set_visible(true);

            title.set_text(&track.title);
            artist_album.set_text(&format!("{} - {}", track.artist, track.album));
            elapsed.set_text(&format_milliseconds(track.elapsed));
            duration.set_text(&format_milliseconds(track.length));

            if let Some(filename) = track.album_art {
                let home = std::env::var("HOME").unwrap();
                let path = format!("{}/.config/rockbox.org/covers/{}", home, filename);
                album_art.set_from_file(Some(&path));
            } else {
                album_art.set_resource(Some("/mg/tsirysndr/Rockbox/icons/jpg/albumart.jpg"));
            }

            let state = self.imp().state.upgrade().unwrap();

            match state.is_liked_track(&track.id) {
                true => self.imp().heart_icon.set_icon_name(Some("heart-symbolic")),
                false => self
                    .imp()
                    .heart_icon
                    .set_icon_name(Some("heart-outline-symbolic")),
            }

            self.imp().set_current_track(Track {
                id: track.id,
                title: track.title,
                artist: track.artist,
                album: track.album,
                album_artist: track.album_artist,
                duration: track.length,
                elapsed: track.elapsed,
                ..Default::default()
            });
        }
    }

    pub fn play(&self) {
        let self_weak = self.downgrade();
        glib::MainContext::default().spawn_local(async move {
            let self_ = match self_weak.upgrade() {
                Some(self_) => self_,
                None => return,
            };
            let rt = tokio::runtime::Runtime::new().unwrap();
            let _ = rt.block_on(async {
                let url = build_url();
                let mut client = PlaybackServiceClient::connect(url).await?;

                match self_.imp().playback_status.get() {
                    1 => {
                        self_
                            .imp()
                            .play_icon
                            .set_icon_name(Some("media-playback-start"));
                        client.pause(PauseRequest {}).await?;
                    }
                    3 => {
                        self_
                            .imp()
                            .play_icon
                            .set_icon_name(Some("media-playback-pause"));
                        client.resume(ResumeRequest {}).await?;
                    }
                    _ => {}
                };
                Ok::<(), Error>(())
            });
        });
    }

    pub fn previous(&self) {
        glib::MainContext::default().spawn_local(async move {
            let rt = tokio::runtime::Runtime::new().unwrap();
            let _ = rt.block_on(async {
                let url = build_url();
                let mut client = PlaybackServiceClient::connect(url).await?;
                client.previous(PreviousRequest {}).await?;
                Ok::<(), Error>(())
            });
        });
    }

    pub fn next(&self) {
        glib::MainContext::default().spawn_local(async move {
            let rt = tokio::runtime::Runtime::new().unwrap();
            let _ = rt.block_on(async {
                let url = build_url();
                let mut client = PlaybackServiceClient::connect(url).await?;
                client.next(NextRequest {}).await?;
                Ok::<(), Error>(())
            });
        });
    }

    pub fn shuffle(&self) {
        let shuffle_enabled = self.imp().shuffle_enabled.get();
        self.imp().shuffle_enabled.set(!shuffle_enabled);
        let shuffle_enabled = self.imp().shuffle_enabled.get();

        match shuffle_enabled {
            true => self
                .imp()
                .shuffle_button
                .remove_css_class("inactive-button"),
            false => self.imp().shuffle_button.add_css_class("inactive-button"),
        }

        glib::MainContext::default().spawn_local(async move {
            let rt = tokio::runtime::Runtime::new().unwrap();
            let _ = rt.block_on(async {
                let url = build_url();
                let mut client = SettingsServiceClient::connect(url).await?;
                client
                    .save_settings(SaveSettingsRequest {
                        playlist_shuffle: Some(shuffle_enabled),
                        ..Default::default()
                    })
                    .await?;
                Ok::<(), Error>(())
            });
        });
    }

    pub fn repeat(&self) {
        let repeat_mode = self.imp().repeat_mode.get();

        match repeat_mode {
            0 => self.imp().repeat_mode.set(3),
            _ => self.imp().repeat_mode.set(0),
        }

        match self.imp().repeat_mode.get() {
            0 => self.imp().repeat_button.add_css_class("inactive-button"),
            _ => self.imp().repeat_button.remove_css_class("inactive-button"),
        }

        let repeat_mode = self.imp().repeat_mode.get();

        glib::MainContext::default().spawn_local(async move {
            let rt = tokio::runtime::Runtime::new().unwrap();
            let _ = rt.block_on(async {
                let url = build_url();
                let mut client = SettingsServiceClient::connect(url).await?;
                let repeat_mode = Some(repeat_mode);
                client
                    .save_settings(SaveSettingsRequest {
                        repeat_mode,
                        ..Default::default()
                    })
                    .await?;
                Ok::<(), Error>(())
            });
        });
    }

    pub fn like(&self) {
        let state = self.imp().state.upgrade().unwrap();
        let track = state.current_track().unwrap();
        let track_id = track.id.clone();
        let heart_icon = self.imp().heart_icon.get();
        let is_liked = state.is_liked_track(&track_id);

        match is_liked {
            true => {
                heart_icon.set_icon_name(Some("heart-outline-symbolic"));
                state.remove_like(&track_id);
            }
            false => {
                heart_icon.set_icon_name(Some("heart-symbolic"));
                state.add_like(track.into());
            }
        }

        thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new().unwrap();
            let url = build_url();
            let _ = rt.block_on(async {
                let mut client = LibraryServiceClient::connect(url).await.unwrap();
                match is_liked {
                    true => {
                        client
                            .unlike_track(UnlikeTrackRequest {
                                id: track_id.clone(),
                            })
                            .await?;
                    }
                    false => {
                        client
                            .like_track(LikeTrackRequest {
                                id: track_id.clone(),
                            })
                            .await?;
                    }
                }

                Ok::<(), Error>(())
            });
        });
    }

    pub fn show_playlist(&self) {
        let playlist_displayed = self.imp().playlist_displayed.get();
        let main_stack = self.imp().main_stack.borrow();
        let library_page = self.imp().library_page.borrow();
        let library_page_ref = library_page.as_ref().unwrap();
        let state = self.imp().state.upgrade().unwrap();
        let playlist_button = self.imp().playlist_button.get();
        let go_back_button = self.imp().go_back_button.borrow();
        let go_back_button_ref = go_back_button.as_ref().unwrap();

        match playlist_displayed {
            true => {
                state.pop_navigation();
                main_stack
                    .as_ref()
                    .unwrap()
                    .set_visible_child_name(&state.current_page().1);
                playlist_button.set_icon_name("chevronup-symbolic");
                playlist_button.set_tooltip_text(Some("Show Play Queue"));
                self.imp().playlist_displayed.set(false);
                go_back_button_ref.set_visible(state.navigation_stack_len() > 1);
                library_page_ref.set_title(&state.current_page().0);
                let current_playlist = self.imp().current_playlist.borrow();
                let current_playlist_ref = current_playlist.as_ref().unwrap();
                current_playlist_ref.hide_top_buttons(false);
                current_playlist_ref.load_current_track();
                current_playlist_ref.load_current_playlist();
                current_playlist_ref.imp().size.set(10);
            }
            false => {
                main_stack
                    .as_ref()
                    .unwrap()
                    .set_visible_child_name("current-playlist-page");
                state.push_navigation("Play Queue", "current-playlist-page");
                playlist_button.set_icon_name("chevrondown-symbolic");
                playlist_button.set_tooltip_text(Some("Hide Play Queue"));
                let current_playlist = self.imp().current_playlist.borrow();
                let current_playlist_ref = current_playlist.as_ref().unwrap();
                current_playlist_ref.hide_top_buttons(true);
                current_playlist_ref.load_current_track();
                current_playlist_ref.load_current_playlist();
                self.imp().playlist_displayed.set(true);
                go_back_button_ref.set_visible(false);
                library_page_ref.set_title("Play Queue");
            }
        }
    }

    pub fn go_to_artist(&self) {
        let state = self.imp().state.upgrade().unwrap();
        let library_page = self.imp().library_page.borrow();
        let main_stack = self.imp().main_stack.borrow();
        let artist_details = self.imp().artist_details.borrow();
        let go_back_button = self.imp().go_back_button.borrow();
        let current_artist_id = self.imp().current_artist_id.borrow();
        let library_page_ref = library_page.as_ref().unwrap();
        let main_stack_ref = main_stack.as_ref().unwrap();
        let go_back_button_ref = go_back_button.as_ref().unwrap();
        let artist_details_ref = artist_details.as_ref().unwrap();
        let current_artist_id_ref = current_artist_id.as_ref().unwrap();
        main_stack_ref.set_visible_child_name("artist-details-page");
        library_page_ref.set_title("Artist");
        go_back_button_ref.set_visible(true);
        artist_details_ref.imp().hide_top_buttons(true);
        artist_details_ref.imp().load_artist(current_artist_id_ref);
        state.push_navigation("Artist", "artist-details-page");
    }

    pub fn go_to_album(&self) {
        let state = self.imp().state.upgrade().unwrap();
        let current_album_id = self.imp().current_album_id.borrow();
        if let Some(album_id) = current_album_id.as_ref() {
            let album_details = self.imp().album_details.borrow();
            let library_page = self.imp().library_page.borrow();
            let main_stack = self.imp().main_stack.borrow();
            let go_back_button = self.imp().go_back_button.borrow();
            let album_details_ref = album_details.as_ref().unwrap();
            let library_page_ref = library_page.as_ref().unwrap();
            let main_stack_ref = main_stack.as_ref().unwrap();
            let go_back_button_ref = go_back_button.as_ref().unwrap();
            main_stack_ref.set_visible_child_name("album-details-page");
            library_page_ref.set_title("Album");
            go_back_button_ref.set_visible(true);
            state.push_navigation("Album", "album-details-page");
            album_details_ref.imp().hide_top_buttons(true);
            album_details_ref.imp().load_album(album_id);
        }
    }
}
