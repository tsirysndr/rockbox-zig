use crate::api::rockbox::v1alpha1::library_service_client::LibraryServiceClient;
use crate::api::rockbox::v1alpha1::playlist_service_client::PlaylistServiceClient;
use crate::api::rockbox::v1alpha1::{
    InsertTracksRequest, LikeTrackRequest, Track, UnlikeTrackRequest,
};
use crate::constants::*;
use crate::state::AppState;
use adw::subclass::prelude::*;
use anyhow::Error;
use glib::subclass;
use gtk::glib;
use gtk::{Button, CompositeTemplate, Image, Label, MenuButton};
use std::cell::RefCell;
use std::env;
use std::thread;

mod imp {

    use super::*;

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(file = "./gtk/song.ui")]
    pub struct Song {
        #[template_child]
        pub album_art_container: TemplateChild<gtk::Box>,
        #[template_child]
        pub album_art: TemplateChild<Image>,
        #[template_child]
        pub track_number: TemplateChild<Label>,
        #[template_child]
        pub track_title: TemplateChild<Label>,
        #[template_child]
        pub artist: TemplateChild<Label>,
        #[template_child]
        pub track_duration: TemplateChild<Label>,
        #[template_child]
        pub heart_button: TemplateChild<Button>,
        #[template_child]
        pub heart_icon: TemplateChild<Image>,
        #[template_child]
        pub more_button: TemplateChild<MenuButton>,

        pub track: RefCell<Option<Track>>,
        pub state: glib::WeakRef<AppState>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for Song {
        const NAME: &'static str = "Song";
        type ParentType = gtk::Box;
        type Type = super::Song;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);

            klass.install_action("app.like-song", None, move |song, _action, _target| {
                song.like();
            });

            klass.install_action("app.play-next", None, move |song, _action, _target| {
                song.play_next();
            });

            klass.install_action("app.play-last", None, move |song, _action, _target| {
                song.play_last();
            });

            klass.install_action("app.add-shuffled", None, move |song, _action, _target| {
                song.add_shuffled();
            });
        }

        fn instance_init(obj: &subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for Song {
        fn constructed(&self) {
            self.parent_constructed();
        }
    }

    impl WidgetImpl for Song {}
    impl BoxImpl for Song {}
}

glib::wrapper! {
  pub struct Song(ObjectSubclass<imp::Song>)
    @extends gtk::Widget, gtk::Box;
}

#[gtk::template_callbacks]
impl Song {
    pub fn new() -> Self {
        glib::Object::new()
    }

    pub fn like(&self) {
        let state = self.imp().state.upgrade().unwrap();
        let track = self.imp().track.borrow().clone().unwrap();
        let heart_icon = self.imp().heart_icon.get();
        let track_id = track.id.clone();
        let is_liked = state.is_liked_track(&track_id);

        match is_liked {
            true => {
                heart_icon.set_icon_name(Some("heart-outline-symbolic"));
                state.remove_like(&track_id);
            }
            false => {
                heart_icon.set_icon_name(Some("heart-symbolic"));
                state.add_like(track.clone());
            }
        }

        thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new().unwrap();
            let url = build_url();
            let result = rt.block_on(async {
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

            match result {
                Ok(_) => {}
                Err(e) => eprintln!("Error liking track: {:?}", e),
            }
        });
    }

    pub fn play_next(&self) {
        let track = self.imp().track.borrow();
        let track = track.as_ref().unwrap();
        let track = track.clone();
        thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new().unwrap();
            let url = build_url();
            let _ = rt.block_on(async {
                let mut client = PlaylistServiceClient::connect(url).await?;
                client
                    .insert_tracks(InsertTracksRequest {
                        tracks: vec![track.path.clone()],
                        position: PLAYLIST_INSERT_FIRST,
                        ..Default::default()
                    })
                    .await?;
                Ok::<(), Error>(())
            });
        });
    }

    pub fn play_last(&self) {
        let track = self.imp().track.borrow();
        let track = track.as_ref().unwrap();
        let track = track.clone();
        thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new().unwrap();
            let url = build_url();
            let _ = rt.block_on(async {
                let mut client = PlaylistServiceClient::connect(url).await?;
                client
                    .insert_tracks(InsertTracksRequest {
                        tracks: vec![track.path.clone()],
                        position: PLAYLIST_INSERT_LAST,
                        ..Default::default()
                    })
                    .await?;
                Ok::<(), Error>(())
            });
        });
    }

    pub fn add_shuffled(&self) {
        let track = self.imp().track.borrow();
        let track = track.as_ref().unwrap();
        let track = track.clone();
        thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new().unwrap();
            let url = build_url();
            let _ = rt.block_on(async {
                let mut client = PlaylistServiceClient::connect(url).await?;
                client
                    .insert_tracks(InsertTracksRequest {
                        tracks: vec![track.path.clone()],
                        position: PLAYLIST_INSERT_SHUFFLED,
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
