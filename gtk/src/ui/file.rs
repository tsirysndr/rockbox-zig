use adw::subclass::prelude::*;
use glib::subclass;
use gtk::glib;
use gtk::{CompositeTemplate, Image, Label, ListBox};
use adw::prelude::*;
use std::cell::RefCell;

mod imp {

    use super::*;

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(file = "./gtk/file.ui")]
    pub struct File {
        #[template_child]
        pub file_icon: TemplateChild<Image>,
        #[template_child]
        pub file_name: TemplateChild<Label>,
        #[template_child]
        pub row: TemplateChild<gtk::Box>,

        pub files: RefCell<Option<ListBox>>,
        pub go_back_button: RefCell<Option<gtk::Button>>,
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

            let self_weak = self.downgrade();
            let click = gtk::GestureClick::new();
            click.connect_released(move |_, _, _, _| {
                if let Some(self_) = self_weak.upgrade() {
                    let files = self_.files.borrow();
                    let files_ref = files.as_ref();
                    let files_ref = files_ref.unwrap();
                    while let Some(file) = files_ref.first_child() {
                        files_ref.remove(&file);
                    }

                    let go_back_button = self_.go_back_button.borrow();
                    if let Some(go_back_button) = go_back_button.as_ref() {
                        go_back_button.set_visible(true);
                    }
                }
            });

            self.row.add_controller(click);
        }
    }

    impl WidgetImpl for File {}
    impl BoxImpl for File {}

    impl File {
        pub fn set_files(&self, files: ListBox) {
            self.files.replace(Some(files));
        }

        pub fn set_go_back_button(&self, go_back_button: Option<gtk::Button>) {
            *self.go_back_button.borrow_mut() = go_back_button;
        }
    }
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

