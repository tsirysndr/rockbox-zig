use crate::api::rockbox::v1alpha1::library_service_client::LibraryServiceClient;
use crate::api::rockbox::v1alpha1::playback_service_client::PlaybackServiceClient;
use crate::api::rockbox::v1alpha1::system_service_client::SystemServiceClient;
use crate::api::rockbox::v1alpha1::{
    GetGlobalStatusRequest, GetGlobalStatusResponse, PlayAllTracksRequest, PlayLikedTracksRequest,
    ScanLibraryRequest, SearchRequest, SearchResponse,
};
use crate::app::RbApplication;
use crate::config;
use crate::state::AppState;
use crate::types::track::Track;
use crate::ui::media_controls::MediaControls;
use crate::ui::pages::album_details::AlbumDetails;
use crate::ui::pages::albums::Albums;
use crate::ui::pages::artist_details::ArtistDetails;
use crate::ui::pages::current_playlist::CurrentPlaylist;
use crate::ui::pages::search::Search;
use crate::ui::pages::songs::Songs;
use crate::ui::pages::{artists::Artists, files::Files, likes::Likes};
use crate::ui::{about_dialog, preferences_dialog};
use adw::prelude::*;
use adw::subclass::prelude::*;
use adw::{
    NavigationPage, NavigationView, OverlaySplitView, StatusPage, TabBar, TabView, ToastOverlay,
    ViewStack, ViewStackPage,
};
use anyhow::Error;
use glib::subclass;
use gtk::{
    gio, glib, Box, Button, CompositeTemplate, ListBox, MenuButton, Overlay, ScrolledWindow,
    SearchBar, SearchEntry, ToggleButton,
};
use preferences_dialog::RbPreferencesDialog;
use std::cell::{Cell, RefCell};
use std::env;
use std::thread;
use tokio::sync::mpsc;

mod imp {
    use super::*;

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(resource = "/io/github/tsirysndr/Rockbox/gtk/window.ui")]
    pub struct RbApplicationWindow {
        #[template_child]
        pub show_sidebar_button: TemplateChild<Button>,
        #[template_child]
        pub primary_menu_button: TemplateChild<MenuButton>,
        #[template_child]
        pub go_back_button: TemplateChild<Button>,
        #[template_child]
        pub play_all_button: TemplateChild<Button>,
        #[template_child]
        pub shuffle_all_button: TemplateChild<Button>,

        #[template_child]
        pub search_bar: TemplateChild<SearchBar>,
        #[template_child]
        pub search_entry: TemplateChild<SearchEntry>,
        #[template_child]
        pub search_button: TemplateChild<ToggleButton>,

        #[template_child]
        pub overlay_split_view: TemplateChild<OverlaySplitView>,
        #[template_child]
        pub navigation_view: TemplateChild<NavigationView>,
        #[template_child]
        pub sidebar_navigation_page: TemplateChild<NavigationPage>,
        #[template_child]
        pub sidebar: TemplateChild<ListBox>,
        #[template_child]
        pub albums_row_box: TemplateChild<Box>,
        #[template_child]
        pub artists_row_box: TemplateChild<Box>,
        #[template_child]
        pub songs_row_box: TemplateChild<Box>,
        #[template_child]
        pub likes_row_box: TemplateChild<Box>,
        #[template_child]
        pub files_row_box: TemplateChild<Box>,

        #[template_child]
        pub toast_overlay: TemplateChild<ToastOverlay>,
        #[template_child]
        pub library_page: TemplateChild<NavigationPage>,
        #[template_child]
        pub main_stack: TemplateChild<ViewStack>,
        #[template_child]
        pub albums_page: TemplateChild<ViewStackPage>,
        #[template_child]
        pub albums_scrolled_window: TemplateChild<ScrolledWindow>,
        #[template_child]
        pub albums: TemplateChild<Albums>,
        #[template_child]
        pub songs_page: TemplateChild<ViewStackPage>,
        #[template_child]
        pub songs_scrolled_window: TemplateChild<ScrolledWindow>,
        #[template_child]
        pub songs: TemplateChild<Songs>,
        #[template_child]
        pub likes_page: TemplateChild<ViewStackPage>,
        #[template_child]
        pub likes_scrolled_window: TemplateChild<ScrolledWindow>,
        #[template_child]
        pub likes: TemplateChild<Likes>,
        #[template_child]
        pub files_page: TemplateChild<ViewStackPage>,
        #[template_child]
        pub files: TemplateChild<Files>,
        #[template_child]
        pub artists_page: TemplateChild<ViewStackPage>,
        #[template_child]
        pub artists_scrolled_window: TemplateChild<ScrolledWindow>,
        #[template_child]
        pub artists: TemplateChild<Artists>,
        #[template_child]
        pub artist_details_page: TemplateChild<ViewStackPage>,
        #[template_child]
        pub artist_details: TemplateChild<ArtistDetails>,
        #[template_child]
        pub artist_tracks_page: TemplateChild<ViewStackPage>,
        #[template_child]
        pub artist_tracks: TemplateChild<Songs>,
        #[template_child]
        pub artist_tracks_scrolled_window: TemplateChild<ScrolledWindow>,
        #[template_child]
        pub album_details_page: TemplateChild<ViewStackPage>,
        #[template_child]
        pub album_details: TemplateChild<AlbumDetails>,
        #[template_child]
        pub search_page: TemplateChild<ViewStackPage>,
        #[template_child]
        pub search: TemplateChild<Search>,
        #[template_child]
        pub current_playlist_page: TemplateChild<ViewStackPage>,
        #[template_child]
        pub current_playlist: TemplateChild<CurrentPlaylist>,
        #[template_child]
        pub library_overlay: TemplateChild<Overlay>,
        #[template_child]
        pub media_control_bar: TemplateChild<MediaControls>,
        #[template_child]
        pub notice_no_results: TemplateChild<StatusPage>,
        #[template_child]
        pub placeholder_page: TemplateChild<ViewStackPage>,

        pub show_sidebar: Cell<bool>,
        pub state: glib::WeakRef<AppState>,
        pub current_track: RefCell<Option<Track>>,
        pub show_placeholder: Cell<bool>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for RbApplicationWindow {
        const NAME: &'static str = "RbApplicationWindow";
        type ParentType = adw::ApplicationWindow;
        type Type = super::RbApplicationWindow;

        fn new() -> Self {
            Self {
                show_sidebar: Cell::new(true),
                state: glib::WeakRef::new(),
                show_placeholder: Cell::new(false),
                ..Default::default()
            }
        }

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);

            klass.install_action("win.show_sidebar", None, move |win, _action, _parameter| {
                let self_ = imp::RbApplicationWindow::from_obj(win);
                self_.toggle_sidebar();
            });

            klass.install_action("win.go_back", None, move |win, _action, _parameter| {
                let self_ = imp::RbApplicationWindow::from_obj(win);
                self_.go_back();
            });

            klass.install_action("app.play_all", None, move |win, _action, _parameter| {
                let self_ = imp::RbApplicationWindow::from_obj(win);
                self_.play_all();
            });

            klass.install_action(
                "app.refresh_library",
                None,
                move |win, _action, _parameter| {
                    let self_ = imp::RbApplicationWindow::from_obj(win);
                    self_.refresh_library();
                },
            );

            klass.install_action("app.shuffle_all", None, move |win, _action, _parameter| {
                let self_ = imp::RbApplicationWindow::from_obj(win);
                self_.shuffle_all();
            });

            klass.install_action("app.preferences", None, move |win, _action, _parameter| {
                let preferences_window = RbPreferencesDialog::default();
                preferences_window.present(Some(win));
            });

            klass.install_action("app.about", None, move |win, _action, _parameter| {
                about_dialog::show(win);
            });

            klass.install_action("app.quit", None, move |win, _action, _parameter| {
                win.close();
            });

            klass.install_action(
                "win.toggle_search",
                None,
                move |win, _action, _parameter| {
                    let self_ = imp::RbApplicationWindow::from_obj(win);
                    self_.toggle_searchbar();
                },
            );

            klass.install_action("app.copy_command", None, move |win, _action, _parameter| {
                const CMD: &str = "rockbox start";
                win.clipboard().set_text(CMD);
                win.add_message_toast("Copied to clipboard");
            });
        }

        fn instance_init(obj: &subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for RbApplicationWindow {
        fn constructed(&self) {
            self.parent_constructed();

            self.verify_rockboxd();

            let weak_self = self.downgrade();
            self.albums_scrolled_window
                .connect_edge_reached(move |_, pos| {
                    if pos == gtk::PositionType::Bottom {
                        let self_ = match weak_self.upgrade() {
                            Some(self_) => self_,
                            None => return,
                        };
                        let size = self_.albums.imp().size.get();
                        let all_albums = self_.albums.imp().all_albums.borrow();
                        let next_range_end = (size + 5).min(all_albums.len());

                        if size >= all_albums.len() {
                            return;
                        }

                        let next_albums = all_albums[size..next_range_end].to_vec();

                        if next_albums.is_empty() {
                            return;
                        }

                        self_.albums.imp().size.set(size + next_albums.len());
                        self_
                            .albums
                            .imp()
                            .create_albums_widgets(Some(next_albums), None);
                    }
                });

            let weak_self = self.downgrade();
            self.artists_scrolled_window
                .connect_edge_reached(move |_, pos| {
                    if pos == gtk::PositionType::Bottom {
                        let self_ = match weak_self.upgrade() {
                            Some(self_) => self_,
                            None => return,
                        };
                        let size = self_.artists.imp().size.get();
                        let all_artists = self_.artists.imp().all_artists.borrow();
                        let next_range_end = (size + 5).min(all_artists.len());

                        if size >= all_artists.len() {
                            return;
                        }

                        let next_artists = all_artists[size..next_range_end].to_vec();

                        if next_artists.is_empty() {
                            return;
                        }

                        self_.artists.imp().size.set(size + next_artists.len());
                        self_
                            .artists
                            .imp()
                            .create_artists_widgets(Some(next_artists), None);
                    }
                });

            let weak_self = self.downgrade();
            self.songs_scrolled_window
                .connect_edge_reached(move |_, pos| {
                    if pos == gtk::PositionType::Bottom {
                        let self_ = match weak_self.upgrade() {
                            Some(self_) => self_,
                            None => return,
                        };
                        let size = self_.songs.imp().size.get();
                        let all_songs = self_.songs.imp().all_tracks.borrow();
                        let next_range_end = (size + 3).min(all_songs.len());

                        if size >= all_songs.len() {
                            return;
                        }

                        let next_songs = all_songs[size..next_range_end].to_vec();

                        if next_songs.is_empty() {
                            return;
                        }

                        self_.songs.imp().size.set(size + next_songs.len());
                        self_
                            .songs
                            .imp()
                            .create_songs_widgets(Some(next_songs), None);
                    }
                });

            let weak_self = self.downgrade();
            self.artist_tracks_scrolled_window
                .connect_edge_reached(move |_, pos| {
                    if pos == gtk::PositionType::Bottom {
                        let self_ = match weak_self.upgrade() {
                            Some(self_) => self_,
                            None => return,
                        };
                        let size = self_.artist_tracks.imp().size.get();
                        let all_songs = self_.artist_tracks.imp().all_tracks.borrow();
                        let next_range_end = (size + 3).min(all_songs.len());

                        if size >= all_songs.len() {
                            return;
                        }

                        let next_songs = all_songs[size..next_range_end].to_vec();

                        if next_songs.is_empty() {
                            return;
                        }

                        self_.artist_tracks.imp().size.set(size + next_songs.len());
                        self_
                            .artist_tracks
                            .imp()
                            .create_songs_widgets(Some(next_songs), None);
                    }
                });

            let weak_self = self.downgrade();
            self.likes_scrolled_window
                .connect_edge_reached(move |_, pos| {
                    if pos == gtk::PositionType::Bottom {
                        let self_ = match weak_self.upgrade() {
                            Some(self_) => self_,
                            None => return,
                        };
                        let size = self_.likes.imp().size.get();
                        let likes = self_.likes.imp().likes.borrow();
                        let next_range_end = (size + 3).min(likes.len());

                        if size >= likes.len() {
                            return;
                        }

                        let next_likes = likes[size..next_range_end].to_vec();

                        if next_likes.is_empty() {
                            return;
                        }

                        self_.likes.imp().size.set(size + next_likes.len());
                        self_
                            .likes
                            .imp()
                            .create_songs_widgets(Some(next_likes), None);
                    }
                });

            let sidebar = self.sidebar.get();
            sidebar.select_row(Some(&sidebar.row_at_index(0).unwrap()));
            let weak_self = self.downgrade();
            sidebar.connect_row_selected(move |_, row| {
                let self_ = match weak_self.upgrade() {
                    Some(self_) => self_,
                    None => return,
                };
                let row = row.unwrap();
                let row = row.clone().downcast::<gtk::ListBoxRow>().unwrap();
                let label = row
                    .child()
                    .unwrap()
                    .downcast::<gtk::Box>()
                    .unwrap()
                    .last_child()
                    .unwrap()
                    .downcast::<gtk::Label>()
                    .unwrap()
                    .text()
                    .to_string();

                if let Some(state) = self_.state.upgrade() {
                    match label.as_str() {
                        "Albums" => {
                            let main_stack = self_.main_stack.get();
                            if !self_.show_placeholder.get() {
                                main_stack.set_visible_child_name("albums-page");
                            }
                            let library_page = self_.library_page.get();
                            library_page.set_title("Albums");
                            state.new_navigation_from("Albums", "albums-page");
                            let play_all_button = self_.play_all_button.get();
                            let shuffle_all_button = self_.shuffle_all_button.get();
                            play_all_button.set_visible(false);
                            shuffle_all_button.set_visible(false);
                        }
                        "Artists" => {
                            let main_stack = self_.main_stack.get();
                            if !self_.show_placeholder.get() {
                                main_stack.set_visible_child_name("artists-page");
                            }
                            let library_page = self_.library_page.get();
                            library_page.set_title("Artists");
                            state.new_navigation_from("Artists", "artists-page");
                            let play_all_button = self_.play_all_button.get();
                            let shuffle_all_button = self_.shuffle_all_button.get();
                            play_all_button.set_visible(false);
                            shuffle_all_button.set_visible(false);
                        }
                        "Songs" => {
                            let main_stack = self_.main_stack.get();
                            if !self_.show_placeholder.get() {
                                main_stack.set_visible_child_name("songs-page");
                            }
                            let library_page = self_.library_page.get();
                            library_page.set_title("Songs");
                            state.new_navigation_from("Songs", "songs-page");
                            let play_all_button = self_.play_all_button.get();
                            let shuffle_all_button = self_.shuffle_all_button.get();

                            if !state.tracks().is_empty() {
                                play_all_button.set_visible(true);
                                shuffle_all_button.set_visible(true);
                            }
                        }
                        "Likes" => {
                            let main_stack = self_.main_stack.get();
                            if !self_.show_placeholder.get() {
                                main_stack.set_visible_child_name("likes-page");
                            }
                            let library_page = self_.library_page.get();
                            library_page.set_title("Likes");
                            state.new_navigation_from("Likes", "likes-page");
                            let play_all_button = self_.play_all_button.get();
                            let shuffle_all_button = self_.shuffle_all_button.get();
                            play_all_button.set_visible(true);
                            shuffle_all_button.set_visible(true);

                            let likes = self_.likes.get();
                            glib::idle_add_local(move || {
                                likes.imp().size.set(20);
                                likes.load_likes();

                                if state.liked_tracks().is_empty() {
                                    play_all_button.set_visible(false);
                                    shuffle_all_button.set_visible(false);
                                }

                                glib::ControlFlow::Break
                            });
                        }
                        "Files" => {
                            let main_stack = self_.main_stack.get();
                            if !self_.show_placeholder.get() {
                                main_stack.set_visible_child_name("files-page");
                            }
                            let library_page = self_.library_page.get();
                            library_page.set_title("Files");
                            state.new_navigation_from("Files", "files-page");
                            let play_all_button = self_.play_all_button.get();
                            let shuffle_all_button = self_.shuffle_all_button.get();
                            play_all_button.set_visible(false);
                            shuffle_all_button.set_visible(false);
                        }
                        _ => {}
                    }

                    let media_control_bar = self_.media_control_bar.get();
                    if media_control_bar.imp().playlist_displayed.get() {
                        media_control_bar.show_playlist();
                    }

                    let search_bar = self_.search_bar.get();
                    search_bar.set_search_mode(false);
                    let state = self_.state.upgrade().unwrap();
                    state.set_search_mode(false);
                }

                let go_back_button = self_.go_back_button.get();
                go_back_button.set_visible(false);
            });
        }
    }

    impl WidgetImpl for RbApplicationWindow {}
    impl WindowImpl for RbApplicationWindow {}
    impl ApplicationWindowImpl for RbApplicationWindow {}
    impl AdwApplicationWindowImpl for RbApplicationWindow {}

    impl RbApplicationWindow {
        fn verify_rockboxd(&self) {
            let rt = tokio::runtime::Runtime::new().unwrap();
            let global_status = rt.block_on(async {
                let url = build_url();
                let mut client = SystemServiceClient::connect(url).await?;
                let response = client.get_global_status(GetGlobalStatusRequest {}).await?;
                Ok::<GetGlobalStatusResponse, Error>(response.into_inner())
            });

            match global_status {
                Ok(_) => {
                    self.show_placeholder.set(false);
                    self.media_control_bar.set_visible(true);
                }
                Err(_) => {
                    let main_stack = self.main_stack.get();
                    main_stack.set_visible_child_name("placeholder-page");
                    self.show_placeholder.set(true);
                    self.media_control_bar.set_visible(false);
                }
            }
        }

        fn toggle_sidebar(&self) {
            let current_state = self.show_sidebar.get();
            self.show_sidebar.set(!current_state);
            self.overlay_split_view.set_show_sidebar(!current_state);
        }

        fn toggle_searchbar(&self) {
            let state = self.state.upgrade().unwrap();
            let search_bar = self.search_bar.get();
            let search_mode = state.search_mode();
            search_bar.set_search_mode(!search_mode);
            state.set_search_mode(!search_mode);
            let search_entry = self.search_entry.get();

            if !search_mode {
                search_entry.grab_focus();

                let main_stack = self.main_stack.get();
                let library_page = self.library_page.get();
                let go_back_button = self.go_back_button.get();
                let state = self.state.upgrade().unwrap();

                if !self.show_placeholder.get() {
                    main_stack.set_visible_child_name("search-page");
                }

                library_page.set_title("Search Results");
                go_back_button.set_visible(true);
                state.push_navigation("Search", "search-page");
                let self_weak = self.downgrade();

                search_entry.connect_changed(move |entry| {
                    let self_ = match self_weak.upgrade() {
                        Some(self_) => self_,
                        None => return,
                    };

                    let text = entry.text();
                    let text = text.to_string();
                    self_.search_term(text.clone());
                });
            } else {
                self.go_back();
            }
        }

        pub fn hide_top_buttons(&self, hide: bool) {
            let play_all_button = self.play_all_button.get();
            let shuffle_all_button = self.shuffle_all_button.get();
            play_all_button.set_visible(!hide);
            shuffle_all_button.set_visible(!hide);
        }

        fn go_back(&self) {
            let main_stack = self.main_stack.get();
            let state = self.state.upgrade().unwrap();

            let poped_page = state.pop_navigation();
            let current_page = state.current_page();

            if current_page.1 == "files-page" && poped_page.1 == "files-page" {
                let files = self.files.get();
                files.go_back();
                return;
            }

            if poped_page.1 == "search-page" {
                let search_bar = self.search_bar.get();
                search_bar.set_search_mode(false);
                state.set_search_mode(false);

                self.search.imp().album_results.clear(false);
                self.search.imp().artist_results.clear(false);
                self.search.imp().track_results.clear(false);
                self.search_entry.get().set_text("");

                if self.show_placeholder.get() {
                    main_stack.set_visible_child_name("placeholder-page");
                }
            }

            if current_page.1 == "search-page" {
                let search_bar = self.search_bar.get();
                search_bar.set_search_mode(true);
                state.set_search_mode(true);
            }

            if !self.show_placeholder.get() {
                main_stack.set_visible_child_name(current_page.1.as_str());
            }

            let library_page = self.library_page.get();
            library_page.set_title(current_page.0.as_str());

            let go_back_button = self.go_back_button.get();
            if state.navigation_stack_len() == 1 {
                go_back_button.set_visible(false);
            }
            if current_page.1 == "files-page"
                && (poped_page.1 == "album-details-page" || poped_page.1 == "artist-details-page")
            {
                let files = self.files.get();
                let default_string = String::from("");
                let state = self.state.upgrade().unwrap();
                let current_path = state.current_path().unwrap_or(String::from(""));
                let music_directory = files.imp().music_directory.borrow();
                let music_directory_ref = music_directory.as_ref().unwrap_or(&default_string);

                go_back_button.set_visible(
                    current_path != *music_directory_ref && current_path != *default_string,
                );
            }

            if current_page.1 == "songs-page" || current_page.1 == "likes-page" {
                self.hide_top_buttons(false);
            } else {
                self.hide_top_buttons(true);
            }
        }

        fn play_all(&self) {
            let state = self.state.upgrade().unwrap();
            if state.current_page().1 == "songs-page" {
                thread::spawn(move || {
                    let rt = tokio::runtime::Runtime::new().unwrap();
                    let url = build_url();
                    let _ = rt.block_on(async {
                        let mut client = PlaybackServiceClient::connect(url).await?;
                        client
                            .play_all_tracks(PlayAllTracksRequest {
                                shuffle: Some(false),
                                position: Some(0),
                            })
                            .await?;
                        Ok::<(), Error>(())
                    });
                });
            }

            if state.current_page().1 == "likes-page" {
                thread::spawn(move || {
                    let rt = tokio::runtime::Runtime::new().unwrap();
                    let url = build_url();
                    let _ = rt.block_on(async {
                        let mut client = PlaybackServiceClient::connect(url).await?;
                        client
                            .play_liked_tracks(PlayLikedTracksRequest {
                                shuffle: Some(false),
                                position: Some(0),
                            })
                            .await?;
                        Ok::<(), Error>(())
                    });
                });
            }
        }

        fn shuffle_all(&self) {
            let state = self.state.upgrade().unwrap();
            if state.current_page().1 == "songs-page" {
                thread::spawn(move || {
                    let rt = tokio::runtime::Runtime::new().unwrap();
                    let url = build_url();
                    let _ = rt.block_on(async {
                        let mut client = PlaybackServiceClient::connect(url).await?;
                        client
                            .play_all_tracks(PlayAllTracksRequest {
                                shuffle: Some(true),
                                position: Some(0),
                            })
                            .await?;
                        Ok::<(), Error>(())
                    });
                });
            }

            if state.current_page().1 == "likes-page" {
                thread::spawn(move || {
                    let rt = tokio::runtime::Runtime::new().unwrap();
                    let url = build_url();
                    let _ = rt.block_on(async {
                        let mut client = PlaybackServiceClient::connect(url).await?;
                        client
                            .play_liked_tracks(PlayLikedTracksRequest {
                                shuffle: Some(true),
                                position: Some(0),
                            })
                            .await?;
                        Ok::<(), Error>(())
                    });
                });
            }
        }

        pub fn refresh_library(&self) {
            let self_weak = self.downgrade();
            let (tx, mut rx) = mpsc::channel(32);
            thread::spawn(move || {
                let rt = tokio::runtime::Runtime::new().unwrap();
                let url = build_url();
                let _ = rt.block_on(async {
                    let mut client = LibraryServiceClient::connect(url).await?;
                    client
                        .scan_library(ScanLibraryRequest { path: None })
                        .await?;
                    tx.send(0).await?;
                    Ok::<(), Error>(())
                });
            });

            let self_ = match self_weak.upgrade() {
                Some(self_) => self_,
                None => return,
            };
            glib::spawn_future_local(async move {
                while let Some(_) = rx.recv().await {
                    let albums = self_.albums.get();
                    albums.imp().size.set(15);
                    albums.load_pictures();
                }
            });
        }

        pub fn search_term(&self, term: String) {
            if term.len() < 3 {
                self.search.imp().album_results.clear(false);
                self.search.imp().artist_results.clear(false);
                self.search.imp().track_results.clear(false);

                return;
            }

            let state = self.state.upgrade().unwrap();
            let albums = state
                .albums()
                .into_iter()
                .filter(|album| {
                    album.title.to_lowercase().contains(&term.to_lowercase())
                        || album.artist.to_lowercase().contains(&term.to_lowercase())
                })
                .collect::<Vec<_>>();

            let artists = state
                .artists()
                .into_iter()
                .filter(|artist| artist.name.to_lowercase().contains(&term.to_lowercase()))
                .collect::<Vec<_>>();

            let tracks = state
                .tracks()
                .into_iter()
                .filter(|track| {
                    track.title.to_lowercase().contains(&term.to_lowercase())
                        || track.artist.to_lowercase().contains(&term.to_lowercase())
                        || track.album.to_lowercase().contains(&term.to_lowercase())
                })
                .take(10)
                .collect::<Vec<_>>();

            state.set_search_results(SearchResponse {
                albums,
                artists,
                tracks,
            });
            self.search.load_results();
        }
    }
}

glib::wrapper! {
    pub struct RbApplicationWindow(
        ObjectSubclass<imp::RbApplicationWindow>)
        @extends gtk::Widget, gtk::Window, gtk::ApplicationWindow, adw::ApplicationWindow,
        @implements gio::ActionMap, gio::ActionGroup;
}

impl RbApplicationWindow {
    pub fn new(state: AppState) -> Self {
        let window: Self = glib::Object::new::<Self>();

        let likes = window.imp().likes.get();
        let main_stack = window.imp().main_stack.get();
        let library_page = window.imp().library_page.get();
        let albums = window.imp().albums.get();
        let album_details = window.imp().album_details.get();
        let artists = window.imp().artists.get();
        let artist_details = window.imp().artist_details.get();
        let files = window.imp().files.get();
        let current_playlist = window.imp().current_playlist.get();
        let media_control_bar = window.imp().media_control_bar.get();
        let go_back_button = window.imp().go_back_button.get();
        let play_all_button = window.imp().play_all_button.get();
        let shuffle_all_button = window.imp().shuffle_all_button.get();
        let songs = window.imp().songs.get();
        let artist_tracks = window.imp().artist_tracks.get();
        let search = window.imp().search.get();
        let album_results = search.imp().album_results.get();
        let artist_results = search.imp().artist_results.get();
        let track_results = search.imp().track_results.get();

        songs.imp().likes_page.replace(Some(likes.clone()));
        track_results.imp().likes_page.replace(Some(likes.clone()));
        artist_tracks.imp().likes_page.replace(Some(likes.clone()));

        window.imp().state.set(Some(&state));
        artists.imp().state.set(Some(&state));
        artist_details.imp().state.set(Some(&state));
        media_control_bar.imp().state.set(Some(&state));
        files.imp().state.set(Some(&state));
        current_playlist.imp().state.set(Some(&state));
        likes.imp().state.set(Some(&state));
        songs.imp().state.set(Some(&state));
        artist_tracks.imp().state.set(Some(&state));
        album_details.imp().state.set(Some(&state));
        artist_results.imp().state.set(Some(&state));
        track_results.imp().state.set(Some(&state));
        album_results.imp().state.set(Some(&state));
        albums.imp().state.set(Some(&state));
        search.imp().state.set(Some(&state));

        artist_results.imp().search_mode.set(true);
        album_results.imp().search_mode.set(true);
        track_results.imp().search_mode.set(true);

        media_control_bar
            .imp()
            .search_bar
            .replace(Some(window.imp().search_bar.get().clone()));

        artists
            .imp()
            .search_bar
            .replace(Some(window.imp().search_bar.get().clone()));

        albums
            .imp()
            .search_bar
            .replace(Some(window.imp().search_bar.get().clone()));

        artist_results
            .imp()
            .search_bar
            .replace(Some(window.imp().search_bar.get().clone()));

        album_results
            .imp()
            .search_bar
            .replace(Some(window.imp().search_bar.get().clone()));

        files.get_music_directory();

        artist_details
            .imp()
            .artist_tracks
            .replace(Some(artist_tracks.clone()));

        artist_details
            .imp()
            .play_all_button
            .replace(Some(play_all_button.clone()));
        artist_details
            .imp()
            .shuffle_all_button
            .replace(Some(shuffle_all_button.clone()));

        album_details
            .imp()
            .play_all_button
            .replace(Some(play_all_button.clone()));
        album_details
            .imp()
            .shuffle_all_button
            .replace(Some(shuffle_all_button.clone()));

        current_playlist
            .imp()
            .play_all_button
            .replace(Some(play_all_button.clone()));
        current_playlist
            .imp()
            .shuffle_all_button
            .replace(Some(shuffle_all_button.clone()));

        media_control_bar
            .imp()
            .library_page
            .replace(Some(library_page.clone()));
        media_control_bar
            .imp()
            .album_details
            .replace(Some(album_details.clone()));
        media_control_bar
            .imp()
            .artist_details
            .replace(Some(artist_details.clone()));
        media_control_bar
            .imp()
            .main_stack
            .replace(Some(main_stack.clone()));
        media_control_bar
            .imp()
            .go_back_button
            .replace(Some(go_back_button.clone()));
        media_control_bar
            .imp()
            .current_playlist
            .replace(Some(current_playlist.clone()));

        albums.imp().set_main_stack(main_stack.clone());
        albums.imp().set_library_page(library_page.clone());
        albums
            .imp()
            .set_go_back_button(window.imp().go_back_button.get().clone());
        albums.imp().set_album_details(album_details.clone());

        artists.imp().set_main_stack(main_stack.clone());
        artists.imp().set_library_page(library_page.clone());
        artists
            .imp()
            .set_go_back_button(window.imp().go_back_button.get().clone());
        artists.imp().set_artist_details(artist_details.clone());
        artists.imp().set_album_details(album_details.clone());

        files.imp().set_main_stack(main_stack.clone());
        files
            .imp()
            .set_go_back_button(window.imp().go_back_button.get().clone());

        album_results.imp().set_main_stack(main_stack.clone());
        album_results.imp().set_library_page(library_page.clone());
        album_results
            .imp()
            .set_go_back_button(window.imp().go_back_button.get().clone());

        artist_results.imp().set_main_stack(main_stack.clone());
        artist_results.imp().set_library_page(library_page.clone());
        artist_results
            .imp()
            .set_go_back_button(window.imp().go_back_button.get().clone());
        artist_results
            .imp()
            .set_artist_details(artist_details.clone());
        artist_results
            .imp()
            .set_album_details(album_details.clone());
        album_results.imp().set_album_details(album_details.clone());

        window
    }

    pub fn add_message_toast(&self, message: &str) {
        let toast = adw::Toast::new(message);
        self.imp().toast_overlay.add_toast(toast);
    }
}

impl Default for RbApplicationWindow {
    fn default() -> Self {
        RbApplication::default()
            .active_window()
            .unwrap()
            .downcast()
            .unwrap()
    }
}

fn build_url() -> String {
    let host = env::var("ROCKBOX_HOST").unwrap_or_else(|_| "localhost".to_string());
    let port = env::var("ROCKBOX_PORT").unwrap_or_else(|_| "6061".to_string());

    format!("tcp://{}:{}", host, port)
}
