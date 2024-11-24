use adw::subclass::prelude::*;
use glib::subclass;
use gtk::glib;
use gtk::CompositeTemplate;

mod imp {

    use super::*;

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(file = "../gtk/likes.ui")]
    pub struct Likes {}

    #[glib::object_subclass]
    impl ObjectSubclass for Likes {
        const NAME: &'static str = "Likes";
        type ParentType = gtk::Box;
        type Type = super::Likes;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
        }

        fn instance_init(obj: &subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for Likes {
        fn constructed(&self) {
            self.parent_constructed();
        }
    }

    impl WidgetImpl for Likes {}
    impl BoxImpl for Likes {}
}

glib::wrapper! {
  pub struct Likes(ObjectSubclass<imp::Likes>)
    @extends gtk::Widget, gtk::Box;
}

#[gtk::template_callbacks]
impl Likes {
    pub fn new() -> Self {
        glib::Object::new()
    }

    pub fn load_likes(&self) {}
}
