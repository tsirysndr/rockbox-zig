use crate::app::RbApplication;
use crate::types::track::Track;
use crate::ui::media_controls::MediaControls;
use crate::ui::pages::album_details::AlbumDetails;
use crate::ui::pages::albums::Albums;
use crate::ui::pages::songs::Songs;
use crate::ui::pages::{artists::Artists, files::Files, likes::Likes};
use adw::prelude::*;
use adw::subclass::prelude::*;
use adw::ViewStack;
use adw::{NavigationPage, NavigationView, OverlaySplitView, ToastOverlay, ViewStackPage};
use glib::subclass;
use gtk::{
    gio, glib, Box, Button, CompositeTemplate, ListBox, MenuButton, Overlay, SearchBar,
    SearchEntry, ToggleButton,
};
use std::cell::RefCell;

mod imp {

    use super::*;

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(file = "gtk/window.ui")]
    pub struct RbApplicationWindow {
        #[template_child]
        pub show_sidebar_button: TemplateChild<Button>,
        #[template_child]
        pub primary_menu_button: TemplateChild<MenuButton>,
        #[template_child]
        pub go_back_button: TemplateChild<Button>,

        #[template_child]
        pub search_bar: TemplateChild<SearchBar>,
        #[template_child]
        pub search_entry: TemplateChild<SearchEntry>,
        #[template_child]
        pub search_button: TemplateChild<ToggleButton>,

        #[template_child]
        pub overlay_split_view: TemplateChild<OverlaySplitView>,
        #[template_child]
        pub navigation_view: TemplateChild<NavigationView>,
        #[template_child]
        pub sidebar_navigation_page: TemplateChild<NavigationPage>,
        #[template_child]
        pub sidebar: TemplateChild<ListBox>,
        #[template_child]
        pub albums_row_box: TemplateChild<Box>,
        #[template_child]
        pub artists_row_box: TemplateChild<Box>,
        #[template_child]
        pub songs_row_box: TemplateChild<Box>,
        #[template_child]
        pub likes_row_box: TemplateChild<Box>,
        #[template_child]
        pub files_row_box: TemplateChild<Box>,

        #[template_child]
        pub toast_overlay: TemplateChild<ToastOverlay>,
        #[template_child]
        pub details_view: TemplateChild<Overlay>,
        #[template_child]
        pub library_page: TemplateChild<NavigationPage>,
        #[template_child]
        pub main_stack: TemplateChild<ViewStack>,
        #[template_child]
        pub albums_page: TemplateChild<ViewStackPage>,
        #[template_child]
        pub albums: TemplateChild<Albums>,
        #[template_child]
        pub songs_page: TemplateChild<ViewStackPage>,
        #[template_child]
        pub songs: TemplateChild<Songs>,
        #[template_child]
        pub likes_page: TemplateChild<ViewStackPage>,
        #[template_child]
        pub likes: TemplateChild<Likes>,
        #[template_child]
        pub files_page: TemplateChild<ViewStackPage>,
        #[template_child]
        pub files: TemplateChild<Files>,
        #[template_child]
        pub artists_page: TemplateChild<ViewStackPage>,
        #[template_child]
        pub artists: TemplateChild<Artists>,
        #[template_child]
        pub album_details_page: TemplateChild<ViewStackPage>,
        #[template_child]
        pub album_details: TemplateChild<AlbumDetails>,
        #[template_child]
        pub library_overlay: TemplateChild<Overlay>,
        #[template_child]
        pub media_control_bar: TemplateChild<MediaControls>,

        pub show_sidebar: std::cell::Cell<bool>,
        pub previous_page: String,
        pub current_track: RefCell<Option<Track>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for RbApplicationWindow {
        const NAME: &'static str = "RbApplicationWindow";
        type ParentType = adw::ApplicationWindow;
        type Type = super::RbApplicationWindow;

        fn new() -> Self {
            Self {
                show_sidebar: std::cell::Cell::new(true),
                previous_page: "Albums".to_string(),
                ..Default::default()
            }
        }

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);

            klass.install_action("win.show_sidebar", None, move |win, _action, _parameter| {
                let self_ = imp::RbApplicationWindow::from_obj(win);
                self_.toggle_sidebar();
            });

            klass.install_action("win.go_back", None, move |win, _action, _parameter| {
                let self_ = imp::RbApplicationWindow::from_obj(win);
                self_.go_back();
                let go_back_button = self_.go_back_button.get();
                go_back_button.set_visible(false);
            });
        }

        fn instance_init(obj: &subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for RbApplicationWindow {
        fn constructed(&self) {
            self.parent_constructed();

            let sidebar = self.sidebar.get();
            sidebar.select_row(Some(&sidebar.row_at_index(0).unwrap()));
            let weak_self = self.downgrade();
            sidebar.connect_row_selected(move |_, row| {
                let self_ = match weak_self.upgrade() {
                    Some(self_) => self_,
                    None => return,
                };
                let row = row.unwrap();
                let row = row.clone().downcast::<gtk::ListBoxRow>().unwrap();
                let label = row
                    .child()
                    .unwrap()
                    .downcast::<gtk::Box>()
                    .unwrap()
                    .last_child()
                    .unwrap()
                    .downcast::<gtk::Label>()
                    .unwrap()
                    .text()
                    .to_string();

                match label.as_str() {
                    "Albums" => {
                        let main_stack = self_.main_stack.get();
                        main_stack.set_visible_child_name("albums-page");
                        let library_page = self_.library_page.get();
                        library_page.set_title("Albums");
                    }
                    "Artists" => {
                        let main_stack = self_.main_stack.get();
                        main_stack.set_visible_child_name("artists-page");
                        let library_page = self_.library_page.get();
                        library_page.set_title("Artists");
                    }
                    "Songs" => {
                        let main_stack = self_.main_stack.get();
                        main_stack.set_visible_child_name("songs-page");
                        let library_page = self_.library_page.get();
                        library_page.set_title("Songs");
                    }
                    "Likes" => {
                        let main_stack = self_.main_stack.get();
                        main_stack.set_visible_child_name("likes-page");
                        let library_page = self_.library_page.get();
                        library_page.set_title("Likes");
                    }
                    "Files" => {
                        let main_stack = self_.main_stack.get();
                        main_stack.set_visible_child_name("files-page");
                        let library_page = self_.library_page.get();
                        library_page.set_title("Files");
                    }
                    _ => {}
                }

                let go_back_button = self_.go_back_button.get();
                go_back_button.set_visible(false);
            });
        }
    }

    impl WidgetImpl for RbApplicationWindow {}
    impl WindowImpl for RbApplicationWindow {}
    impl ApplicationWindowImpl for RbApplicationWindow {}
    impl AdwApplicationWindowImpl for RbApplicationWindow {}

    impl RbApplicationWindow {
        fn toggle_sidebar(&self) {
            let current_state = self.show_sidebar.get();
            self.show_sidebar.set(!current_state);
            self.overlay_split_view.set_show_sidebar(!current_state);
        }

        fn go_back(&self) {
            let main_stack = self.main_stack.get();
            main_stack.set_visible_child_name("albums-page");
            let library_page = self.library_page.get();
            library_page.set_title(self.previous_page.as_str());
        }
    }
}

glib::wrapper! {
    pub struct RbApplicationWindow(
        ObjectSubclass<imp::RbApplicationWindow>)
        @extends gtk::Widget, gtk::Window, gtk::ApplicationWindow, adw::ApplicationWindow,
        @implements gio::ActionMap, gio::ActionGroup;
}

impl RbApplicationWindow {
    pub fn new() -> Self {
        let window: Self = glib::Object::new::<Self>();

        let main_stack = window.imp().main_stack.get();
        let library_page = window.imp().library_page.get();
        let albums = window.imp().albums.get();

        albums.imp().set_main_stack(main_stack.clone());
        albums.imp().set_library_page(library_page.clone());
        albums
            .imp()
            .set_go_back_button(window.imp().go_back_button.get().clone());

        window
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
