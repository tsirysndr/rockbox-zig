use async_graphql::MergedObject;
use browse::BrowseQuery;
use playback::{PlaybackMutation, PlaybackQuery};
use playlist::{PlaylistMutation, PlaylistQuery};
use sound::{SoundMutation, SoundQuery};

pub mod browse;
pub mod metadata;
pub mod objects;
pub mod playback;
pub mod playlist;
pub mod settings;
pub mod sound;
pub mod tagcache;

#[derive(MergedObject, Default)]
pub struct Query(BrowseQuery, PlaybackQuery, PlaylistQuery, SoundQuery);

#[derive(MergedObject, Default)]
pub struct Mutation(PlaybackMutation, PlaylistMutation, SoundMutation);
