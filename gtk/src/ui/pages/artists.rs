use crate::api::rockbox::v1alpha1::library_service_client::LibraryServiceClient;
use crate::api::rockbox::v1alpha1::{GetArtistsRequest, GetArtistsResponse};
use crate::ui::artist::Artist;
use adw::prelude::*;
use adw::subclass::prelude::*;
use anyhow::Error;
use glib::subclass;
use gtk::glib;
use gtk::{CompositeTemplate, FlowBox, Button};
use std::env;
use gtk::pango::EllipsizeMode;
use std::cell::RefCell;

mod imp {

    use super::*;

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(file = "../gtk/artists.ui")]
    pub struct Artists {
        #[template_child]
        pub artists: TemplateChild<FlowBox>,

        pub main_stack: RefCell<Option<adw::ViewStack>>,
        pub library_page: RefCell<Option<adw::NavigationPage>>,
        pub go_back_button: RefCell<Option<Button>>,
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

            let self_weak = self.downgrade();
            glib::idle_add_local(move || {
                let self_ = match self_weak.upgrade() {
                    Some(self_) => self_,
                    None => return glib::ControlFlow::Continue,
                };

                glib::MainContext::default().spawn_local(async move {
                    let obj = self_.obj();
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

        if let Ok(response) = response_{
            let artists = self.imp().artists.clone();
            while let Some(row) = artists.first_child() {
                artists.remove(&row);
            }

            for artist_item in response.artists {
                let artist = Artist::new();
                artist.imp().artist_name.set_text(&artist_item.name);
                artist.imp().artist_name.set_ellipsize(EllipsizeMode::End);
                artist.imp().artist_name.set_max_width_chars(20);

                let main_stack = self.imp().main_stack.borrow().as_ref().unwrap().clone();
                let library_page = self.imp().library_page.borrow().as_ref().unwrap().clone();
                let go_back_button = self.imp().go_back_button.borrow().as_ref().unwrap().clone();

                let click = gtk::GestureClick::new();
                click.connect_released(move |_, _, _, _| {
                    main_stack.set_visible_child_name("artist-details-page");
                    library_page.set_title("Artist");
                    go_back_button.set_visible(true);
                });
                artist.add_controller(click);

                self.imp().artists.append(&artist);
            }
        }
    }
}

fn build_url() -> String {
    let host = env::var("ROCKBOX_HOST").unwrap_or_else(|_| "localhost".to_string());
    let port = env::var("ROCKBOX_PORT").unwrap_or_else(|_| "6061".to_string());
    format!("tcp://{}:{}", host, port)
}
