use adw::subclass::prelude::*;
use glib::subclass;
use gtk::glib;
use gtk::CompositeTemplate;

mod imp {

    use super::*;

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(file = "../gtk/artists.ui")]
    pub struct Artists {}

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
        }
    }

    impl WidgetImpl for Artists {}
    impl BoxImpl for Artists {}
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

    pub fn load_artists(&self) {}
}
