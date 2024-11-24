use adw::subclass::prelude::*;
use glib::subclass;
use gtk::glib;
use gtk::CompositeTemplate;

mod imp {

    use super::*;

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(file = "../gtk/files.ui")]
    pub struct Files {}

    #[glib::object_subclass]
    impl ObjectSubclass for Files {
        const NAME: &'static str = "Files";
        type ParentType = gtk::Box;
        type Type = super::Files;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
        }

        fn instance_init(obj: &subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for Files {
        fn constructed(&self) {
            self.parent_constructed();
        }
    }

    impl WidgetImpl for Files {}
    impl BoxImpl for Files {}
}

glib::wrapper! {
  pub struct Files(ObjectSubclass<imp::Files>)
    @extends gtk::Widget, gtk::Box;
}

#[gtk::template_callbacks]
impl Files {
    pub fn new() -> Self {
        glib::Object::new()
    }

    pub fn load_files(&self, path: Option<&str>) {}
}
