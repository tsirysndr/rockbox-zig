use crate::state::AppState;
use adw::subclass::prelude::*;
use glib::subclass;
use gtk::glib;
use gtk::{Button, CompositeTemplate};
use std::cell::RefCell;

mod imp {

    use super::*;

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(resource = "/io/github/tsirysndr/Rockbox/gtk/playlist_details.ui")]
    pub struct PlaylistDetails {
        pub main_stack: RefCell<Option<adw::ViewStack>>,
        pub go_back_button: RefCell<Option<Button>>,
        pub state: glib::WeakRef<AppState>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for PlaylistDetails {
        const NAME: &'static str = "PlaylistDetails";
        type ParentType = gtk::Box;
        type Type = super::PlaylistDetails;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
        }

        fn instance_init(obj: &subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for PlaylistDetails {
        fn constructed(&self) {
            self.parent_constructed();
        }
    }

    impl WidgetImpl for PlaylistDetails {}
    impl BoxImpl for PlaylistDetails {}
}

glib::wrapper! {
  pub struct PlaylistDetails(ObjectSubclass<imp::PlaylistDetails>)
    @extends gtk::Widget, gtk::Box;
}

#[gtk::template_callbacks]
impl PlaylistDetails {
    pub fn new() -> Self {
        glib::Object::new()
    }
}
