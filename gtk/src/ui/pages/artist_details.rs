use crate::api::rockbox::v1alpha1::library_service_client::LibraryServiceClient;
use crate::api::rockbox::v1alpha1::{GetArtistRequest, GetArtistResponse};
use crate::time::format_milliseconds;
use crate::ui::song::Song;
use adw::prelude::*;
use adw::subclass::prelude::*;
use anyhow::Error;
use glib::subclass;
use gtk::glib;
use gtk::pango::EllipsizeMode;
use gtk::{CompositeTemplate, FlowBox, Image, Label, ListBox, Orientation};
use std::env;

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
    }

    #[glib::object_subclass]
    impl ObjectSubclass for ArtistDetails {
        const NAME: &'static str = "ArtistDetails";
        type ParentType = gtk::Box;
        type Type = super::ArtistDetails;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
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

    pub fn load_artist(&self, id: &str) {
        let id = id.to_string();
        let rt = tokio::runtime::Runtime::new().unwrap();
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
                self.imp().artist_name.set_text(&artist.name);
                let tracks = self.imp().tracks.clone();
                while let Some(row) = tracks.first_child() {
                    tracks.remove(&row);
                }

                for track in artist.tracks.iter().take(10) {
                    let song = Song::new();
                    song.imp().track_title.set_text(&track.title);
                    song.imp().track_number.set_visible(false);
                    song.imp().artist.set_text(&track.artist);
                    song.imp()
                        .track_duration
                        .set_text(&format_milliseconds(track.length as u64));

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

                    song.imp().album_art_container.set_margin_start(15);
                    song.imp().album_art_container.set_visible(true);
                    self.imp().tracks.append(&song);
                }

                let albums = self.imp().albums.clone();
                while let Some(row) = albums.first_child() {
                    albums.remove(&row);
                }

                for album in artist.albums.iter().take(5) {
                    let image = match album.album_art.as_ref() {
                        Some(filename) => {
                            let home = env::var("HOME").unwrap();
                            let path = format!("{}/.config/rockbox.org/covers/{}", home, filename);
                            Image::from_file(&path)
                        }
                        None => {
                            Image::from_resource("/mg/tsirysndr/Rockbox/icons/jpg/albumart.jpg")
                        }
                    };
                    image.set_size_request(200, 200);
                    let image_container = gtk::Box::new(Orientation::Vertical, 0);
                    image_container.append(&image);

                    let title = Label::new(Some(&album.title));
                    title.set_ellipsize(EllipsizeMode::End);
                    title.set_max_width_chars(23);
                    title.add_css_class("album-label");
                    title.set_halign(gtk::Align::Start);

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

                    self.imp().albums.append(&container);
                }
            }
        }
    }
}

fn build_url() -> String {
    let host = env::var("ROCKBOX_HOST").unwrap_or("localhost".to_string());
    let port = env::var("ROCKBOX_PORT").unwrap_or("6061".to_string());
    format!("tcp://{}:{}", host, port)
}
