use adw::subclass::prelude::*;
use glib::subclass;
use gtk::glib;
use gtk::CompositeTemplate;

mod imp {

    use super::*;

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(file = "../gtk/artist_details.ui")]
    pub struct ArtistDetails {}

    #[glib::object_subclass]
    impl ObjectSubclass for ArtistDetails {
        const NAME: &'static str = "ArtistDetails";
        type ParentType = gtk::Box;
        type Type = super::ArtistDetails;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
        }

        fn instance_init(obj: &subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for ArtistDetails {
        fn constructed(&self) {
            self.parent_constructed();
        }
    }

    impl WidgetImpl for ArtistDetails {}
    impl BoxImpl for ArtistDetails {}
}

glib::wrapper! {
  pub struct ArtistDetails(ObjectSubclass<imp::ArtistDetails>)
    @extends gtk::Widget, gtk::Box;
}

#[gtk::template_callbacks]
impl ArtistDetails {
    pub fn new() -> Self {
        glib::Object::new()
    }

    pub fn load_artist(&self, id: &str) {}
}
