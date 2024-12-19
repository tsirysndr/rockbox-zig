use crate::ui::pages::albums::Albums;
use crate::ui::pages::artists::Artists;
use crate::ui::pages::songs::Songs;
use adw::prelude::*;
use adw::subclass::prelude::*;
use glib::subclass;
use gtk::pango::EllipsizeMode;
use gtk::prelude::WidgetExt;
use gtk::CompositeTemplate;
use gtk::{glib, Box, Button, ScrolledWindow};
use std::cell::{Cell, RefCell};

mod imp {
    use crate::state::AppState;

    use super::*;

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(resource = "/io/github/tsirysndr/Rockbox/gtk/search.ui")]
    pub struct Search {
        #[template_child]
        pub albums: TemplateChild<Button>,
        #[template_child]
        pub artists: TemplateChild<Button>,
        #[template_child]
        pub tracks: TemplateChild<Button>,

        #[template_child]
        pub albums_scrolled_window: TemplateChild<ScrolledWindow>,
        #[template_child]
        pub album_results: TemplateChild<Albums>,

        #[template_child]
        pub artists_scrolled_window: TemplateChild<ScrolledWindow>,
        #[template_child]
        pub artist_results: TemplateChild<Artists>,

        #[template_child]
        pub tracks_scrolled_window: TemplateChild<ScrolledWindow>,
        #[template_child]
        pub track_results: TemplateChild<Songs>,

        pub current_tab: Cell<u32>,
        pub state: glib::WeakRef<AppState>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for Search {
        const NAME: &'static str = "Search";
        type ParentType = gtk::Box;
        type Type = super::Search;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);

            klass.install_action(
                "app.on_activate_albums",
                None,
                move |search, _action, _parameter| {
                    search.imp().albums.remove_css_class("tab");
                    search.imp().artists.add_css_class("tab");
                    search.imp().tracks.add_css_class("tab");

                    search.imp().albums_scrolled_window.set_visible(true);
                    search.imp().artists_scrolled_window.set_visible(false);
                    search.imp().tracks_scrolled_window.set_visible(false);
                    search.imp().current_tab.set(0);

                    let state = search.imp().state.upgrade().unwrap();
                    if let Some(results) = state.search_results() {
                        search
                            .imp()
                            .album_results
                            .load_search_results(results.albums);
                    }
                },
            );

            klass.install_action(
                "app.on_activate_artists",
                None,
                move |search, _action, _parameter| {
                    search.imp().artists.remove_css_class("tab");
                    search.imp().albums.add_css_class("tab");
                    search.imp().tracks.add_css_class("tab");

                    search.imp().artists_scrolled_window.set_visible(true);
                    search.imp().albums_scrolled_window.set_visible(false);
                    search.imp().tracks_scrolled_window.set_visible(false);
                    search.imp().current_tab.set(1);

                    let state = search.imp().state.upgrade().unwrap();
                    if let Some(results) = state.search_results() {
                        search
                            .imp()
                            .artist_results
                            .load_search_results(results.artists);
                    }
                },
            );

            klass.install_action(
                "app.on_activate_tracks",
                None,
                move |search, _action, _parameter| {
                    search.imp().tracks.remove_css_class("tab");
                    search.imp().albums.add_css_class("tab");
                    search.imp().artists.add_css_class("tab");

                    search.imp().tracks_scrolled_window.set_visible(true);
                    search.imp().albums_scrolled_window.set_visible(false);
                    search.imp().artists_scrolled_window.set_visible(false);
                    search.imp().current_tab.set(2);

                    let state = search.imp().state.upgrade().unwrap();
                    if let Some(results) = state.search_results() {
                        search
                            .imp()
                            .track_results
                            .load_search_results(results.tracks);
                    }
                },
            );
        }

        fn instance_init(obj: &subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for Search {
        fn constructed(&self) {
            self.parent_constructed();
            self.current_tab.set(0);
        }
    }

    impl WidgetImpl for Search {}
    impl BoxImpl for Search {}
}

glib::wrapper! {
  pub struct Search(ObjectSubclass<imp::Search>)
    @extends gtk::Widget, gtk::Box;
}

#[gtk::template_callbacks]
impl Search {
    pub fn new() -> Self {
        glib::Object::new()
    }

    pub fn load_results(&self) {
        let state = self.imp().state.upgrade().unwrap();
        let results = state.search_results().unwrap();

        match self.imp().current_tab.get() {
            0 => {
                self.imp().album_results.load_search_results(results.albums);
            }
            1 => {
                self.imp()
                    .artist_results
                    .load_search_results(results.artists);
            }
            2 => {
                self.imp().track_results.load_search_results(results.tracks);
            }
            _ => {}
        }
    }
}
