use crate::api::rockbox::v1alpha1::Album as RockboxAlbum;
use crate::api::rockbox::v1alpha1::Artist as RockboxArtist;
use crate::api::rockbox::v1alpha1::SearchResponse;
use crate::api::rockbox::v1alpha1::Track as RockboxTrack;
use crate::navigation::NavigationHistory;
use crate::types::track::Track;
use glib::subclass::prelude::*;
use gtk::glib;
use std::cell::{Cell, RefCell};
use std::collections::HashMap;

mod imp {

    use super::*;

    #[derive(Default)]
    pub struct AppState {
        pub navigation_history: RefCell<NavigationHistory>,
        pub current_path: RefCell<Option<String>>,
        pub music_directory: RefCell<Option<String>>,
        pub current_track: RefCell<Option<Track>>,
        pub likes: RefCell<HashMap<String, RockboxTrack>>,
        pub resume_index: Cell<i32>,
        pub resume_elapsed: Cell<u32>,
        pub search_mode: Cell<bool>,
        pub search_results: RefCell<Option<SearchResponse>>,
        pub albums: RefCell<Vec<RockboxAlbum>>,
        pub tracks: RefCell<Vec<RockboxTrack>>,
        pub artists: RefCell<Vec<RockboxArtist>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for AppState {
        const NAME: &'static str = "AppState";
        type Type = super::AppState;
        type ParentType = glib::Object;
    }

    impl ObjectImpl for AppState {}
}

glib::wrapper! {
    pub struct AppState(ObjectSubclass<imp::AppState>);
}

impl AppState {
    pub fn new() -> Self {
        let obj = glib::Object::new::<Self>();

        obj.imp()
            .navigation_history
            .replace(NavigationHistory::new());
        obj.imp().current_path.replace(None);
        obj.imp().music_directory.replace(None);
        obj.imp().current_track.replace(None);
        obj.imp().likes.replace(HashMap::new());
        obj.imp().resume_index.set(-1);
        obj.imp().resume_elapsed.set(0);
        obj.imp().search_mode.set(false);
        obj.imp().search_results.replace(None);
        obj.imp().albums.replace(Vec::new());
        obj.imp().artists.replace(Vec::new());
        obj.imp().tracks.replace(Vec::new());
        obj
    }

    pub fn new_navigation_from(&self, page_name: &str, page_id: &str) {
        let self_ = imp::AppState::from_obj(self);
        *self_.navigation_history.borrow_mut() =
            NavigationHistory::new_from(page_name.to_string(), page_id.to_string());
    }

    pub fn push_navigation(&self, page_name: &str, page_id: &str) {
        let self_ = imp::AppState::from_obj(self);
        let navigation = self_.navigation_history.borrow_mut();
        navigation.push(page_name.to_string(), page_id.to_string());
    }

    pub fn pop_navigation(&self) -> (String, String) {
        let self_ = imp::AppState::from_obj(self);
        let navigation = self_.navigation_history.borrow_mut();
        navigation.pop()
    }

    pub fn current_page(&self) -> (String, String) {
        let self_ = imp::AppState::from_obj(self);
        let navigation = self_.navigation_history.borrow();
        navigation.current()
    }

    pub fn reset_navigation(&self) {
        let self_ = imp::AppState::from_obj(self);
        let navigation = self_.navigation_history.borrow_mut();
        navigation.reset();
    }

    pub fn navigation_stack_len(&self) -> usize {
        let self_ = imp::AppState::from_obj(self);
        let navigation = self_.navigation_history.borrow();
        navigation.len()
    }

    pub fn get_navigation_history(&self) -> NavigationHistory {
        let self_ = imp::AppState::from_obj(self);
        self_.navigation_history.borrow().clone()
    }

    pub fn current_path(&self) -> Option<String> {
        let self_ = imp::AppState::from_obj(self);
        self_.current_path.borrow().clone()
    }

    pub fn music_directory(&self) -> Option<String> {
        let self_ = imp::AppState::from_obj(self);
        self_.music_directory.borrow().clone()
    }

    pub fn set_current_path(&self, path: &str) {
        let self_ = imp::AppState::from_obj(self);
        *self_.current_path.borrow_mut() = Some(path.to_string());
    }

    pub fn set_music_directory(&self, path: &str) {
        let self_ = imp::AppState::from_obj(self);
        *self_.music_directory.borrow_mut() = Some(path.to_string());
    }

    pub fn current_track(&self) -> Option<Track> {
        let self_ = imp::AppState::from_obj(self);
        self_.current_track.borrow().clone()
    }

    pub fn set_current_track(&self, track: Track) {
        let self_ = imp::AppState::from_obj(self);
        *self_.current_track.borrow_mut() = Some(track);
    }

    pub fn set_liked_tracks(&self, liked_tracks: Vec<RockboxTrack>) {
        let self_ = imp::AppState::from_obj(self);
        let mut likes = self_.likes.borrow_mut();
        likes.clear();
        for like in liked_tracks {
            likes.insert(like.id.clone(), like);
        }
    }

    pub fn liked_tracks(&self) -> Vec<RockboxTrack> {
        let self_ = imp::AppState::from_obj(self);
        let likes = self_.likes.borrow();
        likes.values().cloned().collect()
    }

    pub fn is_liked_track(&self, track_id: &str) -> bool {
        let self_ = imp::AppState::from_obj(self);
        let likes = self_.likes.borrow();
        likes.contains_key(track_id)
    }

    pub fn remove_like(&self, track_id: &str) {
        let self_ = imp::AppState::from_obj(self);
        let mut likes = self_.likes.borrow_mut();
        likes.remove(track_id);
    }

    pub fn add_like(&self, track: RockboxTrack) {
        let self_ = imp::AppState::from_obj(self);
        let mut likes = self_.likes.borrow_mut();
        likes.insert(track.id.clone(), track);
    }

    pub fn resume_index(&self) -> i32 {
        let self_ = imp::AppState::from_obj(self);
        self_.resume_index.get()
    }

    pub fn resume_elapsed(&self) -> u32 {
        let self_ = imp::AppState::from_obj(self);
        self_.resume_elapsed.get()
    }

    pub fn set_resume_index(&self, index: i32) {
        let self_ = imp::AppState::from_obj(self);
        self_.resume_index.set(index);
    }

    pub fn set_resume_elapsed(&self, elapsed: u32) {
        let self_ = imp::AppState::from_obj(self);
        self_.resume_elapsed.set(elapsed);
    }

    pub fn search_mode(&self) -> bool {
        let self_ = imp::AppState::from_obj(self);
        self_.search_mode.get()
    }

    pub fn set_search_mode(&self, mode: bool) {
        let self_ = imp::AppState::from_obj(self);
        self_.search_mode.set(mode);
    }

    pub fn search_results(&self) -> Option<SearchResponse> {
        let self_ = imp::AppState::from_obj(self);
        self_.search_results.borrow().clone()
    }

    pub fn set_search_results(&self, results: SearchResponse) {
        let self_ = imp::AppState::from_obj(self);
        self_.search_results.replace(Some(results));
    }

    pub fn clear_search_results(&self) {
        let self_ = imp::AppState::from_obj(self);
        self_
            .search_results
            .replace(Some(SearchResponse::default()));
    }

    pub fn set_albums(&self, albums: Vec<RockboxAlbum>) {
        let self_ = imp::AppState::from_obj(self);
        *self_.albums.borrow_mut() = albums;
    }

    pub fn albums(&self) -> Vec<RockboxAlbum> {
        let self_ = imp::AppState::from_obj(self);
        let albums = self_.albums.borrow();
        albums.clone()
    }

    pub fn set_artists(&self, artists: Vec<RockboxArtist>) {
        let self_ = imp::AppState::from_obj(self);
        *self_.artists.borrow_mut() = artists;
    }

    pub fn artists(&self) -> Vec<RockboxArtist> {
        let self_ = imp::AppState::from_obj(self);
        let artists = self_.artists.borrow();
        artists.clone()
    }

    pub fn set_tracks(&self, tracks: Vec<RockboxTrack>) {
        let self_ = imp::AppState::from_obj(self);
        *self_.tracks.borrow_mut() = tracks;
    }

    pub fn tracks(&self) -> Vec<RockboxTrack> {
        let self_ = imp::AppState::from_obj(self);
        let tracks = self_.tracks.borrow();
        tracks.clone()
    }
}
