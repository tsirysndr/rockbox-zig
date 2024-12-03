use adw::subclass::prelude::*;
use glib::subclass;
use gtk::glib;
use gtk::{Button, CompositeTemplate, Image, Label};

mod imp {

    use super::*;

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(file = "./gtk/song.ui")]
    pub struct Song {
        #[template_child]
        pub album_art_container: TemplateChild<gtk::Box>,
        #[template_child]
        pub album_art: TemplateChild<Image>,
        #[template_child]
        pub track_number: TemplateChild<Label>,
        #[template_child]
        pub track_title: TemplateChild<Label>,
        #[template_child]
        pub artist: TemplateChild<Label>,
        #[template_child]
        pub track_duration: TemplateChild<Label>,
        #[template_child]
        pub heart_button: TemplateChild<Button>,
        #[template_child]
        pub heart_icon: TemplateChild<Image>,
        #[template_child]
        pub more_button: TemplateChild<Button>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for Song {
        const NAME: &'static str = "Song";
        type ParentType = gtk::Box;
        type Type = super::Song;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);

            klass.install_action("app.like-song", None, move |_song, _action, _target| {});
        }

        fn instance_init(obj: &subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for Song {
        fn constructed(&self) {
            self.parent_constructed();
        }
    }

    impl WidgetImpl for Song {}
    impl BoxImpl for Song {}
}

glib::wrapper! {
  pub struct Song(ObjectSubclass<imp::Song>)
    @extends gtk::Widget, gtk::Box;
}

#[gtk::template_callbacks]
impl Song {
    pub fn new() -> Self {
        glib::Object::new()
    }
}
