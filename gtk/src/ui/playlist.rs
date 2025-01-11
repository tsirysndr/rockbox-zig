use crate::state::AppState;
use crate::ui::pages::playlist_details::PlaylistDetails;
use crate::ui::{delete_playlist::DeletePlaylistDialog, edit_playlist::EditPlaylistDialog};
use adw::prelude::*;
use adw::subclass::prelude::*;
use glib::subclass;
use gtk::glib;
use gtk::{Button, CompositeTemplate, Image, Label, MenuButton};
use std::cell::RefCell;

mod imp {

    use crate::ui::pages::playlist_details;

    use super::*;

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(resource = "/io/github/tsirysndr/Rockbox/gtk/playlist.ui")]
    pub struct Playlist {
        #[template_child]
        pub playlist_icon: TemplateChild<Image>,
        #[template_child]
        pub playlist_name: TemplateChild<Label>,
        #[template_child]
        pub row: TemplateChild<gtk::Box>,
        #[template_child]
        pub playlist_menu: TemplateChild<MenuButton>,

        pub main_stack: RefCell<Option<adw::ViewStack>>,
        pub go_back_button: RefCell<Option<Button>>,
        pub state: glib::WeakRef<AppState>,
        pub playlist_details: RefCell<Option<PlaylistDetails>>,
        pub playlist_id: RefCell<Option<String>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for Playlist {
        const NAME: &'static str = "Playlist";
        type ParentType = gtk::Box;
        type Type = super::Playlist;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);

            klass.install_action(
                "app.playlist.play-now",
                None,
                move |playlist, _action, _target| {
                    playlist.play_now(false);
                },
            );

            klass.install_action(
                "app.playlist.play-next",
                None,
                move |playlist, _action, _target| {
                    playlist.play_next();
                },
            );

            klass.install_action(
                "app.playlist.play-last",
                None,
                move |playlist, _action, _target| {
                    playlist.play_next();
                },
            );

            klass.install_action(
                "app.playlist.add-shuffled",
                None,
                move |playlist, _action, _target| {
                    playlist.add_shuffled();
                },
            );

            klass.install_action(
                "app.playlist.play-last-shuffled",
                None,
                move |playlist, _action, _target| {
                    playlist.play_last_shuffled();
                },
            );

            klass.install_action(
                "app.playlist.play-shuffled",
                None,
                move |playlist, _action, _target| {
                    playlist.play_now(true);
                },
            );

            klass.install_action(
                "app.playlist.edit",
                None,
                move |playlist, _action, _target| {
                    let edit_playlist_dialog = EditPlaylistDialog::default();
                    let self_ = imp::Playlist::from_obj(playlist);
                    let state = self_.state.upgrade().unwrap();
                    let playlist_id = self_.playlist_id.borrow();
                    state.set_selected_playlist(playlist_id.clone());
                    edit_playlist_dialog.imp().state.set(Some(&state));
                    edit_playlist_dialog.present(Some(playlist));
                },
            );

            klass.install_action(
                "app.playlist.delete",
                None,
                move |playlist, _action, _target| {
                    let delete_playlist_dialog = DeletePlaylistDialog::default();
                    let self_ = imp::Playlist::from_obj(playlist);
                    let state = self_.state.upgrade().unwrap();
                    let playlist_id = self_.playlist_id.borrow();
                    state.set_selected_playlist(playlist_id.clone());
                    delete_playlist_dialog.imp().state.set(Some(&state));
                    delete_playlist_dialog.present(Some(playlist));
                },
            );
        }

        fn instance_init(obj: &subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for Playlist {
        fn constructed(&self) {
            self.parent_constructed();
            let self_weak = self.downgrade();
            let click = gtk::GestureClick::new();
            click.connect_released(move |_, _, _, _| {
                if let Some(self_) = self_weak.upgrade() {
                    let main_stack = self_.main_stack.borrow();
                    let main_stack_ref = main_stack.as_ref().unwrap();
                    main_stack_ref.set_visible_child_name("playlist-details-page");
                    let state = self_.state.upgrade().unwrap();
                    state.push_navigation("Playlist", "playlist-details-page");

                    let go_back_button = self_.go_back_button.borrow();
                    if let Some(go_back_button) = go_back_button.as_ref() {
                        go_back_button.set_visible(true);
                    }
                    let playlist_details = self_.playlist_details.borrow();
                    if let Some(playlist_details) = playlist_details.as_ref() {
                        let playlist_id = self_.playlist_id.borrow();
                        let playlist_id = playlist_id.as_ref().unwrap();
                        playlist_details.load_tracks(playlist_id.clone());
                    }
                }
            });

            self.row.add_controller(click);
        }
    }

    impl WidgetImpl for Playlist {}
    impl BoxImpl for Playlist {}
}

glib::wrapper! {
  pub struct Playlist(ObjectSubclass<imp::Playlist>)
    @extends gtk::Widget, gtk::Box;
}

#[gtk::template_callbacks]
impl Playlist {
    pub fn new() -> Self {
        glib::Object::new()
    }

    pub fn play_now(&self, _shuffle: bool) {}

    pub fn play_next(&self) {}

    pub fn play_last(&self) {}

    pub fn add_shuffled(&self) {}

    pub fn play_last_shuffled(&self) {}
}
