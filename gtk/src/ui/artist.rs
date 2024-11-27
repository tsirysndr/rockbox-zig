use adw::subclass::prelude::*;
use glib::subclass;
use gtk::glib;
use gtk::{CompositeTemplate, Image, Label};

mod imp {

    use super::*;

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(file = "./gtk/artist.ui")]
    pub struct Artist {
        #[template_child]
        pub artist_image: TemplateChild<Image>,
        #[template_child]
        pub artist_noimage: TemplateChild<gtk::Box>,
        #[template_child]
        pub artist_name: TemplateChild<Label>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for Artist {
        const NAME: &'static str = "Artist";
        type ParentType = gtk::Box;
        type Type = super::Artist;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
        }

        fn instance_init(obj: &subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for Artist {
        fn constructed(&self) {
            self.parent_constructed();
        }
    }

    impl WidgetImpl for Artist {}
    impl BoxImpl for Artist {}
}

glib::wrapper! {
  pub struct Artist(ObjectSubclass<imp::Artist>)
    @extends gtk::Widget, gtk::Box;
}

#[gtk::template_callbacks]
impl Artist {
    pub fn new() -> Self {
        glib::Object::new()
    }
}
