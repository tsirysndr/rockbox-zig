use adw::subclass::prelude::*;
use glib::subclass;
use gtk::glib;
use gtk::CompositeTemplate;

mod imp {

    use super::*;

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(file = "../gtk/album_details.ui")]
    pub struct AlbumDetails {}

    #[glib::object_subclass]
    impl ObjectSubclass for AlbumDetails {
        const NAME: &'static str = "AlbumDetails";
        type ParentType = gtk::Box;
        type Type = super::AlbumDetails;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
        }

        fn instance_init(obj: &subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for AlbumDetails {
        fn constructed(&self) {
            self.parent_constructed();
        }
    }

    impl WidgetImpl for AlbumDetails {}
    impl BoxImpl for AlbumDetails {}
}

glib::wrapper! {
  pub struct AlbumDetails(ObjectSubclass<imp::AlbumDetails>)
    @extends gtk::Widget, gtk::Box;
}

#[gtk::template_callbacks]
impl AlbumDetails {
    pub fn new() -> Self {
        glib::Object::new()
    }

    pub fn load_album(&self, id: &str) {}
}
