use async_graphql::{MergedObject, MergedSubscription};
use browse::BrowseQuery;
use library::LibraryQuery;
use playback::{PlaybackMutation, PlaybackQuery, PlaybackSubscription};
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

#[derive(MergedObject, Default)]
pub struct Query(
    BrowseQuery,
    LibraryQuery,
    PlaybackQuery,
    PlaylistQuery,
    SoundQuery,
    SettingsQuery,
    SystemQuery,
);

#[derive(MergedObject, Default)]
pub struct Mutation(PlaybackMutation, PlaylistMutation, SoundMutation);

#[derive(MergedSubscription, Default)]
pub struct Subscription(PlaybackSubscription);
