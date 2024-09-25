use async_graphql::MergedObject;
use browse::BrowseQuery;
use playback::{PlaybackMutation, PlaybackQuery};
use playlist::{PlaylistMutation, PlaylistQuery};
use settings::SettingsQuery;
use sound::{SoundMutation, SoundQuery};
use system::SystemQuery;

pub mod browse;
pub mod library;
pub mod metadata;
pub mod objects;
pub mod playback;
pub mod playlist;
pub mod settings;
pub mod sound;
pub mod system;
pub mod tagcache;

#[derive(MergedObject, Default)]
pub struct Query(
    BrowseQuery,
    PlaybackQuery,
    PlaylistQuery,
    SoundQuery,
    SettingsQuery,
    SystemQuery,
);

#[derive(MergedObject, Default)]
pub struct Mutation(PlaybackMutation, PlaylistMutation, SoundMutation);
