use crate::state::AppState;
use adw::subclass::prelude::*;
use glib::subclass;
use gtk::glib;
use gtk::{Button, CompositeTemplate, Image, Label, MenuButton};
use std::cell::RefCell;

mod imp {

    use super::*;

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(resource = "/io/github/tsirysndr/Rockbox/gtk/playlist_folder.ui")]
    pub struct PlaylistFolder {
        #[template_child]
        pub folder_icon: TemplateChild<Image>,
        #[template_child]
        pub folder_name: TemplateChild<Label>,
        #[template_child]
        pub row: TemplateChild<gtk::Box>,
        #[template_child]
        pub folder_menu: TemplateChild<MenuButton>,

        pub main_stack: RefCell<Option<adw::ViewStack>>,
        pub go_back_button: RefCell<Option<Button>>,
        pub state: glib::WeakRef<AppState>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for PlaylistFolder {
        const NAME: &'static str = "PlaylistFolder";
        type ParentType = gtk::Box;
        type Type = super::PlaylistFolder;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);

            klass.install_action(
                "app.folder.rename",
                None,
                move |folder, _action, _target| {
                    folder.rename();
                },
            );

            klass.install_action(
                "app.folder.delete",
                None,
                move |folder, _action, _target| {
                    folder.delete();
                },
            );

            klass.install_action(
                "app.folder.create-playlist",
                None,
                move |folder, _action, _target| {
                    folder.create_playlist();
                },
            );
        }

        fn instance_init(obj: &subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for PlaylistFolder {
        fn constructed(&self) {
            self.parent_constructed();
        }
    }

    impl WidgetImpl for PlaylistFolder {}
    impl BoxImpl for PlaylistFolder {}
}

glib::wrapper! {
  pub struct PlaylistFolder(ObjectSubclass<imp::PlaylistFolder>)
    @extends gtk::Widget, gtk::Box;
}

#[gtk::template_callbacks]
impl PlaylistFolder {
    pub fn new() -> Self {
        glib::Object::new()
    }

    pub fn rename(&self) {}

    pub fn delete(&self) {}

    pub fn create_playlist(&self) {}
}
