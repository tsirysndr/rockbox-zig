use adw::subclass::prelude::*;
use glib::subclass;
use gtk::glib;
use gtk::{CompositeTemplate, Image, Label};

mod imp {

    use super::*;

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(file = "./gtk/file.ui")]
    pub struct File {
        #[template_child]
        pub file_icon: TemplateChild<Image>,
        #[template_child]
        pub file_name: TemplateChild<Label>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for File {
        const NAME: &'static str = "File";
        type ParentType = gtk::Box;
        type Type = super::File;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
        }

        fn instance_init(obj: &subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for File {
        fn constructed(&self) {
            self.parent_constructed();
        }
    }

    impl WidgetImpl for File {}
    impl BoxImpl for File {}
}

glib::wrapper! {
  pub struct File(ObjectSubclass<imp::File>)
    @extends gtk::Widget, gtk::Box;
}

#[gtk::template_callbacks]
impl File {
    pub fn new() -> Self {
        glib::Object::new()
    }
}
