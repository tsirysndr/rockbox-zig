use crate::api::rockbox::v1alpha1::library_service_client::LibraryServiceClient;
use crate::api::rockbox::v1alpha1::{
    Artist as ArtistItem, GetArtistsRequest, GetArtistsResponse, SearchRequest,
};
use crate::state::AppState;
use crate::ui::artist::Artist;
use crate::ui::pages::album_details::AlbumDetails;
use crate::ui::pages::artist_details::ArtistDetails;
use adw::prelude::*;
use adw::subclass::prelude::*;
use anyhow::Error;
use glib::subclass;
use gtk::glib;
use gtk::pango::EllipsizeMode;
use gtk::{Button, CompositeTemplate, FlowBox, SearchBar};
use std::cell::{Cell, RefCell};
use std::{env, thread};

mod imp {

    use super::*;

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(resource = "/mg/tsirysndr/Rockbox/gtk/artists.ui")]
    pub struct Artists {
        #[template_child]
        pub artists: TemplateChild<FlowBox>,

        pub main_stack: RefCell<Option<adw::ViewStack>>,
        pub library_page: RefCell<Option<adw::NavigationPage>>,
        pub go_back_button: RefCell<Option<Button>>,
        pub artist_details: RefCell<Option<ArtistDetails>>,
        pub album_details: RefCell<Option<AlbumDetails>>,
        pub state: glib::WeakRef<AppState>,
        pub size: Cell<usize>,
        pub all_artists: RefCell<Vec<ArtistItem>>,
        pub search_mode: Cell<bool>,
        pub search_bar: RefCell<Option<SearchBar>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for Artists {
        const NAME: &'static str = "Artists";
        type ParentType = gtk::Box;
        type Type = super::Artists;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
        }

        fn instance_init(obj: &subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for Artists {
        fn constructed(&self) {
            self.parent_constructed();

            self.size.set(30);

            let self_weak = self.downgrade();
            glib::idle_add_local(move || {
                let self_ = match self_weak.upgrade() {
                    Some(self_) => self_,
                    None => return glib::ControlFlow::Continue,
                };

                glib::spawn_future_local(async move {
                    let obj = self_.obj();

                    if obj.imp().search_mode.get() {
                        return;
                    }

                    obj.load_artists();
                });

                glib::ControlFlow::Break
            });
        }
    }

    impl WidgetImpl for Artists {}
    impl BoxImpl for Artists {}

    impl Artists {
        pub fn set_main_stack(&self, main_stack: adw::ViewStack) {
            *self.main_stack.borrow_mut() = Some(main_stack);
        }

        pub fn set_library_page(&self, library_page: adw::NavigationPage) {
            *self.library_page.borrow_mut() = Some(library_page);
        }

        pub fn set_go_back_button(&self, go_back_button: Button) {
            *self.go_back_button.borrow_mut() = Some(go_back_button);
        }

        pub fn set_artist_details(&self, artist_details: ArtistDetails) {
            *self.artist_details.borrow_mut() = Some(artist_details);
            let artist_details = self.artist_details.borrow_mut();
            if let Some(artist_details) = artist_details.as_ref() {
                let main_stack = self.main_stack.borrow();
                let main_stack = main_stack.as_ref().unwrap();
                let library_page = self.library_page.borrow();
                let library_page = library_page.as_ref().unwrap();
                artist_details.imp().set_main_stack(main_stack.clone());
                artist_details.imp().set_library_page(library_page.clone());
            }
        }

        pub fn set_album_details(&self, album_details: AlbumDetails) {
            *self.album_details.borrow_mut() = Some(album_details);
            let artist_details = self.artist_details.borrow();
            let artist_details = artist_details.as_ref().unwrap();
            let album_details = self.album_details.borrow();
            let album_details = album_details.as_ref().unwrap();
            artist_details
                .imp()
                .set_album_details(album_details.clone());
        }

        pub fn create_artists_widgets(&self, list: Option<Vec<ArtistItem>>, limit: Option<usize>) {
            let artists = self.all_artists.borrow();
            let artists = match list {
                Some(list) => list.clone().into_iter().take(list.len()),
                None => artists
                    .clone()
                    .into_iter()
                    .take(limit.unwrap_or(artists.len())),
            };
            for artist_item in artists {
                let artist = Artist::new();
                artist.imp().artist_name.set_text(&artist_item.name);
                artist.imp().artist_name.set_ellipsize(EllipsizeMode::End);
                artist.imp().artist_name.set_max_width_chars(40);

                let main_stack = self.main_stack.borrow().as_ref().unwrap().clone();
                let library_page = self.library_page.borrow().as_ref().unwrap().clone();
                let go_back_button = self.go_back_button.borrow().as_ref().unwrap().clone();
                let artist_details = self.artist_details.borrow().as_ref().unwrap().clone();
                let search_bar = self.search_bar.borrow().as_ref().unwrap().clone();
                let state = self.state.upgrade().unwrap();
                let artist_id = artist_item.id.clone();

                let click = gtk::GestureClick::new();
                click.connect_released(move |_, _, _, _| {
                    main_stack.set_visible_child_name("artist-details-page");
                    library_page.set_title("Artist");
                    go_back_button.set_visible(true);
                    artist_details.imp().load_artist(&artist_id);
                    state.push_navigation("Artist", "artist-details-page");
                    search_bar.set_search_mode(false);
                    state.set_search_mode(false);
                });
                artist.add_controller(click);

                self.artists.append(&artist);
            }
        }
    }
}

glib::wrapper! {
  pub struct Artists(ObjectSubclass<imp::Artists>)
    @extends gtk::Widget, gtk::Box;
}

#[gtk::template_callbacks]
impl Artists {
    pub fn new() -> Self {
        glib::Object::new()
    }

    pub fn load_artists(&self) {
        let rt = tokio::runtime::Runtime::new().unwrap();
        let response_ = rt.block_on(async {
            let url = build_url();
            let mut client = LibraryServiceClient::connect(url).await?;
            let response = client.get_artists(GetArtistsRequest {}).await?.into_inner();
            Ok::<GetArtistsResponse, Error>(response)
        });

        if let Ok(response) = response_ {
            let state = self.imp().state.upgrade().unwrap();
            state.set_artists(response.artists.clone());

            let artists = self.imp().artists.clone();
            while let Some(row) = artists.first_child() {
                artists.remove(&row);
            }

            self.imp().all_artists.replace(response.artists.clone());
            self.imp().create_artists_widgets(None, Some(40));
        }
    }

    pub fn clear(&self, ui_only: bool) {
        if !ui_only {
            let state = self.imp().state.upgrade().unwrap();
            state.clear_search_results();
        }

        let artists_ = self.imp().artists.clone();
        while let Some(row) = artists_.first_child() {
            artists_.remove(&row);
        }
    }

    pub fn load_search_results(&self, artists: Vec<ArtistItem>) {
        self.clear(true);

        self.imp().all_artists.replace(artists.clone());
        self.imp()
            .create_artists_widgets(Some(artists.clone()), None);
    }
}

fn build_url() -> String {
    let host = env::var("ROCKBOX_HOST").unwrap_or_else(|_| "localhost".to_string());
    let port = env::var("ROCKBOX_PORT").unwrap_or_else(|_| "6061".to_string());
    format!("tcp://{}:{}", host, port)
}
