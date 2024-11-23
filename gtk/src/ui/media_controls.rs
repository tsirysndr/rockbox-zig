use adw::subclass::prelude::*;
use glib::subclass;
use gtk::glib;
use gtk::{Button, CompositeTemplate, Image};

mod imp {

    use super::*;

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(file = "gtk/media_controls.ui")]
    pub struct MediaControls {
        #[template_child]
        pub shuffle_button: TemplateChild<Button>,
        #[template_child]
        pub previous_button: TemplateChild<Button>,
        #[template_child]
        pub play_pause_button: TemplateChild<Button>,
        #[template_child]
        pub next_button: TemplateChild<Button>,
        #[template_child]
        pub repeat_button: TemplateChild<Button>,
        #[template_child]
        pub album_art: TemplateChild<Image>,
        #[template_child]
        pub current_song_details: TemplateChild<gtk::Box>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for MediaControls {
        const NAME: &'static str = "MediaControls";
        type ParentType = gtk::Box;
        type Type = super::MediaControls;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
        }

        fn instance_init(obj: &subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for MediaControls {}
    impl WidgetImpl for MediaControls {}
    impl BoxImpl for MediaControls {}
}

glib::wrapper! {
  pub struct MediaControls(ObjectSubclass<imp::MediaControls>)
    @extends gtk::Widget, adw::Bin;
}

#[gtk::template_callbacks]
impl MediaControls {
    pub fn new() -> Self {
        glib::Object::new()
    }
}
