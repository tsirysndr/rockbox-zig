use std::{env, thread};

use crate::api::rockbox::v1alpha1::library_service_client::LibraryServiceClient;
use crate::api::rockbox::v1alpha1::{Album, GetAlbumsRequest, GetAlbumsResponse, SearchRequest};
use crate::state::AppState;
use crate::ui::pages::album_details::AlbumDetails;
use adw::prelude::*;
use adw::subclass::prelude::*;
use anyhow::Error;
use glib::subclass;
use gtk::pango::EllipsizeMode;
use gtk::prelude::WidgetExt;
use gtk::CompositeTemplate;
use gtk::{glib, Box, FlowBox, Image, Label, Orientation, SearchBar};
use std::cell::{Cell, RefCell};

mod imp {

    use super::*;

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(resource = "/mg/tsirysndr/Rockbox/gtk/albums.ui")]
    pub struct Albums {
        #[template_child]
        pub library: TemplateChild<FlowBox>,
        pub main_stack: RefCell<Option<adw::ViewStack>>,
        pub library_page: RefCell<Option<adw::NavigationPage>>,
        pub go_back_button: RefCell<Option<gtk::Button>>,
        pub album_details: RefCell<Option<AlbumDetails>>,
        pub previous_page: RefCell<Vec<(String, String)>>,
        pub all_albums: RefCell<Vec<Album>>,
        pub size: Cell<usize>,
        pub search_mode: Cell<bool>,
        pub state: glib::WeakRef<AppState>,
        pub search_bar: RefCell<Option<SearchBar>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for Albums {
        const NAME: &'static str = "Albums";
        type ParentType = gtk::Box;
        type Type = super::Albums;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
        }

        fn instance_init(obj: &subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for Albums {
        fn constructed(&self) {
            self.parent_constructed();

            self.size.set(15);

            let self_weak = self.downgrade();
            glib::idle_add_local(move || {
                let self_ = match self_weak.upgrade() {
                    Some(self_) => self_,
                    None => return glib::ControlFlow::Continue,
                };

                glib::MainContext::default().spawn_local(async move {
                    let obj = self_.obj();

                    if obj.imp().search_mode.get() {
                        return;
                    }

                    obj.load_pictures();
                });

                glib::ControlFlow::Break
            });
        }
    }
    impl WidgetImpl for Albums {}
    impl BoxImpl for Albums {}

    impl Albums {
        pub fn set_main_stack(&self, main_stack: adw::ViewStack) {
            *self.main_stack.borrow_mut() = Some(main_stack);
        }

        pub fn set_library_page(&self, library_page: adw::NavigationPage) {
            *self.library_page.borrow_mut() = Some(library_page);
        }

        pub fn set_go_back_button(&self, go_back_button: gtk::Button) {
            *self.go_back_button.borrow_mut() = Some(go_back_button);
        }

        pub fn set_album_details(&self, album_details: AlbumDetails) {
            *self.album_details.borrow_mut() = Some(album_details);
        }

        pub fn set_previous_page(&self, previous_page: Vec<(String, String)>) {
            *self.previous_page.borrow_mut() = previous_page;
        }

        pub fn add_picture_to_library(
            &self,
            id: &str,
            filename: Option<String>,
            title: &str,
            artist: &str,
            year: u32,
        ) {
            let home = std::env::var("HOME").unwrap();
            let image = match filename {
                Some(filename) => {
                    Image::from_file(&format!("{}/.config/rockbox.org/covers/{}", home, filename))
                }
                None => Image::from_resource("/mg/tsirysndr/Rockbox/icons/jpg/albumart.jpg"),
            };
            image.set_size_request(200, 200);
            let image_container = Box::new(Orientation::Vertical, 0);

            let self_weak = self.downgrade();
            let album_id = id.to_string();

            let gesture = gtk::GestureClick::new();
            gesture.connect_released(move |_, _, _, _| {
                let self_ = match self_weak.upgrade() {
                    Some(self_) => self_,
                    None => return,
                };
                let obj = self_.obj();
                obj.navigate_to_details(&album_id);
                obj.imp()
                    .album_details
                    .borrow()
                    .as_ref()
                    .unwrap()
                    .imp()
                    .load_album(&album_id);
            });

            image_container.append(&image);
            image_container.add_controller(gesture);
            image_container.add_css_class("rounded-image");

            let self_weak = self.downgrade();
            let album_id = id.to_string();
            let label_click = gtk::GestureClick::new();
            label_click.connect_released(move |_, _, _, _| {
                let self_ = match self_weak.upgrade() {
                    Some(self_) => self_,
                    None => return,
                };
                let obj = self_.obj();
                obj.navigate_to_details(&album_id);
                obj.imp()
                    .album_details
                    .borrow()
                    .as_ref()
                    .unwrap()
                    .imp()
                    .load_album(&album_id);
            });

            let title = Label::new(Some(title));
            title.set_ellipsize(EllipsizeMode::End);
            title.set_max_width_chars(23);
            title.add_css_class("album-label");
            title.set_halign(gtk::Align::Start);
            title.add_controller(label_click);

            let artist = Label::new(Some(artist));
            artist.set_ellipsize(EllipsizeMode::End);
            artist.set_max_width_chars(23);
            artist.set_halign(gtk::Align::Start);

            let year_u32 = year;
            let year = Label::new(Some(&year.to_string()));
            year.set_halign(gtk::Align::Start);
            year.add_css_class("year-label");

            let infos = Box::new(Orientation::Vertical, 4);
            infos.append(&title);
            infos.append(&artist);

            if year_u32 != 0 {
                infos.append(&year);
            }

            infos.set_halign(gtk::Align::Start);

            let container = Box::new(Orientation::Vertical, 10);
            container.append(&image_container);
            container.append(&infos);
            container.set_margin_bottom(56);

            self.library.append(&container);
        }

        pub fn create_albums_widgets(&self, list: Option<Vec<Album>>, limit: Option<usize>) {
            let albums = self.all_albums.borrow();
            let albums = match list {
                Some(list) => list.clone().into_iter().take(list.len()),
                None => albums
                    .clone()
                    .into_iter()
                    .take(limit.unwrap_or(albums.len())),
            };
            for album in albums {
                self.add_picture_to_library(
                    &album.id,
                    album.album_art,
                    &album.title,
                    &album.artist,
                    album.year,
                );
            }
        }
    }
}

glib::wrapper! {
  pub struct Albums(ObjectSubclass<imp::Albums>)
    @extends gtk::Widget, gtk::Box;
}

#[gtk::template_callbacks]
impl Albums {
    pub fn new() -> Self {
        glib::Object::new()
    }

    pub fn navigate_to_details(&self, _title: &str) {
        if let Some(main_stack) = self.imp().main_stack.borrow().as_ref() {
            main_stack.set_visible_child_name("album-details-page");
            if let Some(library_page) = self.imp().library_page.borrow().as_ref() {
                library_page.set_title("Album");
            }

            if let Some(go_back_button) = self.imp().go_back_button.borrow().as_ref() {
                go_back_button.set_visible(true);
            }

            let state = self.imp().state.upgrade().unwrap();
            match self.imp().search_mode.get() {
                true => state.push_navigation("Search Results", "search-page"),
                false => state.push_navigation("Albums", "albums-page"),
            }

            let search_bar = self.imp().search_bar.borrow().as_ref().unwrap().clone();
            search_bar.set_search_mode(false);
            state.set_search_mode(false);
        } else {
            eprintln!("NavigationView not set!");
        }
    }

    pub fn load_pictures(&self) {
        let handle = thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                let host = env::var("ROCKBOX_HOST").unwrap_or_else(|_| "localhost".to_string());
                let port = env::var("ROCKBOX_PORT").unwrap_or_else(|_| "6061".to_string());

                let url = format!("tcp://{}:{}", host, port);
                let mut client = LibraryServiceClient::connect(url).await?;
                let request = tonic::Request::new(GetAlbumsRequest {});
                let response = client.get_albums(request).await?;
                Ok::<GetAlbumsResponse, Error>(response.into_inner())
            })
        });

        if let Ok(response) = handle.join().unwrap() {
            let state = self.imp().state.upgrade().unwrap();
            state.set_albums(response.albums.clone());

            self.clear(true);

            self.imp().all_albums.replace(response.albums.clone());
            self.imp().create_albums_widgets(None, Some(15));
        }
    }

    pub fn clear(&self, ui_only: bool) {
        if !ui_only {
            let state = self.imp().state.upgrade().unwrap();
            state.clear_search_results();
        }

        let library = self.imp().library.get();
        while let Some(child) = library.first_child() {
            library.remove(&child);
        }
    }

    pub fn load_search_results(&self, albums: Vec<Album>) {
        self.clear(true);

        self.imp().all_albums.replace(albums.clone());
        self.imp().create_albums_widgets(Some(albums.clone()), None);
    }
}

fn build_url() -> String {
    let host = env::var("ROCKBOX_HOST").unwrap_or_else(|_| "localhost".to_string());
    let port = env::var("ROCKBOX_PORT").unwrap_or_else(|_| "6061".to_string());
    format!("tcp://{}:{}", host, port)
}
