use crate::state::AppState;
use adw::subclass::prelude::*;
use glib::subclass;
use gtk::glib;
use gtk::{Button, CompositeTemplate, Image, Label, MenuButton};
use std::cell::RefCell;

mod imp {

    use super::*;

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(resource = "/io/github/tsirysndr/Rockbox/gtk/playlist.ui")]
    pub struct Playlist {
        #[template_child]
        pub playlist_icon: TemplateChild<Image>,
        #[template_child]
        pub playlist_name: TemplateChild<Label>,
        #[template_child]
        pub row: TemplateChild<gtk::Box>,
        #[template_child]
        pub playlist_menu: TemplateChild<MenuButton>,

        pub main_stack: RefCell<Option<adw::ViewStack>>,
        pub go_back_button: RefCell<Option<Button>>,
        pub state: glib::WeakRef<AppState>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for Playlist {
        const NAME: &'static str = "Playlist";
        type ParentType = gtk::Box;
        type Type = super::Playlist;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
        }

        fn instance_init(obj: &subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for Playlist {
        fn constructed(&self) {
            self.parent_constructed();
        }
    }

    impl WidgetImpl for Playlist {}
    impl BoxImpl for Playlist {}
}

glib::wrapper! {
  pub struct Playlist(ObjectSubclass<imp::Playlist>)
    @extends gtk::Widget, gtk::Box;
}

#[gtk::template_callbacks]
impl Playlist {
    pub fn new() -> Self {
        glib::Object::new()
    }
}
