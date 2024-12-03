use crate::api::rockbox::v1alpha1::library_service_client::LibraryServiceClient;
use crate::api::rockbox::v1alpha1::{GetLikedTracksRequest, GetLikedTracksResponse, Track};
use crate::state::AppState;
use crate::time::format_milliseconds;
use crate::ui::song::Song;
use adw::prelude::*;
use adw::subclass::prelude::*;
use anyhow::Error;
use glib::subclass;
use gtk::glib;
use gtk::pango::EllipsizeMode;
use gtk::{CompositeTemplate, ListBox};
use std::cell::{Cell, RefCell};
use std::env;

mod imp {

    use super::*;

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(file = "../gtk/likes.ui")]
    pub struct Likes {
        #[template_child]
        pub tracks: TemplateChild<ListBox>,

        pub likes: RefCell<Vec<Track>>,
        pub size: Cell<usize>,
        pub state: glib::WeakRef<AppState>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for Likes {
        const NAME: &'static str = "Likes";
        type ParentType = gtk::Box;
        type Type = super::Likes;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
        }

        fn instance_init(obj: &subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for Likes {
        fn constructed(&self) {
            self.parent_constructed();

            self.size.set(20);

            let self_weak = self.downgrade();
            glib::idle_add_local(move || {
                let self_ = match self_weak.upgrade() {
                    Some(self_) => self_,
                    None => return glib::ControlFlow::Continue,
                };

                glib::MainContext::default().spawn_local(async move {
                    let obj = self_.obj();
                    obj.load_likes();
                });
                glib::ControlFlow::Break
            });
        }
    }

    impl WidgetImpl for Likes {}
    impl BoxImpl for Likes {}

    impl Likes {
        pub fn create_songs_widgets(&self, list: Option<Vec<Track>>, limit: Option<usize>) {
            let likes = self.likes.borrow();
            let likes = match list {
                Some(list) => {
                    let cloned_list = list.clone();
                    cloned_list.into_iter().enumerate().take(list.len())
                }
                None => {
                    let cloned_likes = likes.clone();
                    cloned_likes
                        .into_iter()
                        .enumerate()
                        .take(limit.unwrap_or(likes.len()))
                }
            };

            let size = self.size.get();

            for (index, track) in likes {
                let song = Song::new();
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

                song.imp().heart_icon.set_icon_name(Some("heart-symbolic"));

                match track.album_art.as_ref() {
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
                self.tracks.append(&song);
            }
        }
    }
}

glib::wrapper! {
  pub struct Likes(ObjectSubclass<imp::Likes>)
    @extends gtk::Widget, gtk::Box;
}

#[gtk::template_callbacks]
impl Likes {
    pub fn new() -> Self {
        glib::Object::new()
    }

    pub fn load_likes(&self) {
        let rt = tokio::runtime::Runtime::new().unwrap();
        let response_ = rt.block_on(async {
            let url = build_url();
            let mut client = LibraryServiceClient::connect(url).await?;
            let response = client
                .get_liked_tracks(GetLikedTracksRequest {})
                .await?
                .into_inner();
            Ok::<GetLikedTracksResponse, Error>(response)
        });

        if let Ok(response) = response_ {
            let tracks = self.imp().tracks.clone();
            while let Some(row) = tracks.first_child() {
                tracks.remove(&row);
            }

            self.imp().likes.replace(response.tracks.clone());

            let state = self.imp().state.upgrade().unwrap();
            state.set_liked_tracks(response.tracks.clone());

            self.imp().create_songs_widgets(None, Some(20));
        }
    }
}

fn build_url() -> String {
    let host = env::var("ROCKBOX_HOST").unwrap_or_else(|_| "localhost".to_string());
    let port = env::var("ROCKBOX_PORT").unwrap_or_else(|_| "6061".to_string());
    format!("tcp://{}:{}", host, port)
}
