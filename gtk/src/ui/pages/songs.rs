use crate::api::rockbox::v1alpha1::library_service_client::LibraryServiceClient;
use crate::api::rockbox::v1alpha1::{GetTracksRequest, GetTracksResponse};
use crate::time::format_milliseconds;
use crate::ui::song::Song;
use adw::prelude::*;
use adw::subclass::prelude::*;
use anyhow::Error;
use glib::subclass;
use gtk::glib;
use gtk::{CompositeTemplate, ListBox};
use std::env;

mod imp {

    use super::*;

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(file = "../gtk/songs.ui")]
    pub struct Songs {
        #[template_child]
        pub tracks: TemplateChild<ListBox>,
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

            let self_weak = self.downgrade();
            glib::idle_add_local(move || {
                let self_ = match self_weak.upgrade() {
                    Some(self_) => self_,
                    None => return glib::ControlFlow::Continue,
                };

                glib::MainContext::default().spawn_local(async move {
                    let obj = self_.obj();
                    obj.load_songs();
                });
                glib::ControlFlow::Break
            });
        }
    }

    impl WidgetImpl for Songs {}
    impl BoxImpl for Songs {}
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
            let tracks = self.imp().tracks.clone();
            while let Some(row) = tracks.first_child() {
                tracks.remove(&row);
            }

            for (index, track) in response.tracks.iter().enumerate().take(20) {
                let song = Song::new();
                song.imp().track_number.set_text(&format!("{}", index + 1));
                song.imp().track_title.set_text(&track.title);
                song.imp().artist.set_text(&track.artist);
                song.imp()
                    .track_duration
                    .set_text(&format!("{}", format_milliseconds(track.length as u64)));

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
                self.imp().tracks.append(&song);
            }
        }
    }
}

fn build_url() -> String {
    let host = env::var("ROCKBOX_HOST").unwrap_or_else(|_| "localhost".to_string());
    let port = env::var("ROCKBOX_PORT").unwrap_or_else(|_| "6061".to_string());
    format!("tcp://{}:{}", host, port)
}
