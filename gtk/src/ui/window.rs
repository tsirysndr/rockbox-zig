use adw::prelude::*;
use adw::subclass::prelude::*;
use glib::subclass;
use gtk::{gio, glib, CompositeTemplate};

use crate::app::RbApplication;

mod imp {

    use super::*;

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(file = "window.ui")]
    pub struct RbApplicationWindow {}

    #[glib::object_subclass]
    impl ObjectSubclass for RbApplicationWindow {
        const NAME: &'static str = "RbApplicationWindow";
        type ParentType = adw::ApplicationWindow;
        type Type = super::RbApplicationWindow;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
        }

        fn instance_init(obj: &subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for RbApplicationWindow {}

    impl WidgetImpl for RbApplicationWindow {}
    impl WindowImpl for RbApplicationWindow {}
    impl ApplicationWindowImpl for RbApplicationWindow {}
    impl AdwApplicationWindowImpl for RbApplicationWindow {}
    impl RbApplicationWindow {}
}

glib::wrapper! {
    pub struct RbApplicationWindow(
        ObjectSubclass<imp::RbApplicationWindow>)
        @extends gtk::Widget, gtk::Window, gtk::ApplicationWindow, adw::ApplicationWindow,
        @implements gio::ActionMap, gio::ActionGroup;
}

impl RbApplicationWindow {
    pub fn new() -> Self {
        glib::Object::new::<Self>()
    }
}

impl Default for RbApplicationWindow {
    fn default() -> Self {
        RbApplication::default()
            .active_window()
            .unwrap()
            .downcast()
            .unwrap()
    }
}
