use adw::subclass::prelude::*;
use glib::subclass;
use gtk::glib;
use gtk::{CompositeTemplate, ListBox};

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

    pub fn load_songs(&self) {}
}
