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
    #[template(file = "../gtk/current_playlist.ui")]
    pub struct CurrentPlaylist {}

    #[glib::object_subclass]
    impl ObjectSubclass for CurrentPlaylist {
        const NAME: &'static str = "CurrentPlaylist";
        type ParentType = gtk::Box;
        type Type = super::CurrentPlaylist;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
        }

        fn instance_init(obj: &subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for CurrentPlaylist {
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
                    // obj.load_songs();
                });
                glib::ControlFlow::Break
            });
        }
    }

    impl WidgetImpl for CurrentPlaylist {}
    impl BoxImpl for CurrentPlaylist {}
}

glib::wrapper! {
  pub struct CurrentPlaylist(ObjectSubclass<imp::CurrentPlaylist>)
    @extends gtk::Widget, gtk::Box;
}

#[gtk::template_callbacks]
impl CurrentPlaylist {
    pub fn new() -> Self {
        glib::Object::new()
    }

    pub fn load_playlist(&self) {
        let rt = tokio::runtime::Runtime::new().unwrap();
    }
}

fn build_url() -> String {
    let host = env::var("ROCKBOX_HOST").unwrap_or_else(|_| "localhost".to_string());
    let port = env::var("ROCKBOX_PORT").unwrap_or_else(|_| "6061".to_string());
    format!("tcp://{}:{}", host, port)
}
