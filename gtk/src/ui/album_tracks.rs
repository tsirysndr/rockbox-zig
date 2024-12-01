use crate::api::rockbox::v1alpha1::Track;
use crate::time::format_milliseconds;
use crate::ui::song::Song;
use adw::prelude::*;
use adw::subclass::prelude::*;
use anyhow::Error;
use glib::subclass;
use gtk::glib;
use gtk::{CompositeTemplate, Image, Label, ListBox};
use std::{env, thread};

mod imp {

    use super::*;

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(file = "./gtk/album_tracks.ui")]
    pub struct AlbumTracks {
        #[template_child]
        pub volume: TemplateChild<Label>,
        #[template_child]
        pub tracklist: TemplateChild<ListBox>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for AlbumTracks {
        const NAME: &'static str = "AlbumTracks";
        type ParentType = gtk::Box;
        type Type = super::AlbumTracks;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
        }

        fn instance_init(obj: &subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for AlbumTracks {
        fn constructed(&self) {
            self.parent_constructed();
        }
    }

    impl WidgetImpl for AlbumTracks {}
    impl BoxImpl for AlbumTracks {}

    impl AlbumTracks {
        pub fn load_tracks(&self, tracks: Vec<Track>, disc: Option<u32>) {
            if let Some(disc) = disc {
                self.volume.set_text(&format!("Volume {}", disc));
                self.volume.set_visible(true);
            } else {
                self.volume.set_visible(false);
            }
            let tracklist = self.tracklist.clone();
            while let Some(row) = tracklist.first_child() {
                tracklist.remove(&row);
            }

            for track in tracks {
                let song = Song::new();
                song.imp()
                    .track_number
                    .set_text(&format!("{:02}", track.track_number));
                song.imp().track_title.set_text(&track.title);
                song.imp().artist.set_text(&track.artist);
                song.imp()
                    .track_duration
                    .set_text(&format_milliseconds(track.length as u64));
                self.tracklist.append(&song);
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
  pub struct AlbumTracks(ObjectSubclass<imp::AlbumTracks>)
    @extends gtk::Widget, gtk::Box;
}

#[gtk::template_callbacks]
impl AlbumTracks {
    pub fn new() -> Self {
        glib::Object::new()
    }
}
