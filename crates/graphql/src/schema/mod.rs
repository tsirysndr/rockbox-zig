use async_graphql::{MergedObject, MergedSubscription};
use browse::BrowseQuery;
use device::{DeviceMutation, DeviceQuery};
use library::{LibraryMutation, LibraryQuery};
use playback::{PlaybackMutation, PlaybackQuery, PlaybackSubscription};
use playlist::{PlaylistMutation, PlaylistQuery, PlaylistSubscription};
use settings::{SettingsMutation, SettingsQuery};
use sound::{SoundMutation, SoundQuery};
use system::SystemQuery;

pub mod browse;
pub mod device;
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
    DeviceQuery,
    LibraryQuery,
    PlaybackQuery,
    PlaylistQuery,
    SoundQuery,
    SettingsQuery,
    SystemQuery,
);

#[derive(MergedObject, Default)]
pub struct Mutation(
    DeviceMutation,
    PlaybackMutation,
    PlaylistMutation,
    SoundMutation,
    LibraryMutation,
    SettingsMutation,
);

#[derive(MergedSubscription, Default)]
pub struct Subscription(PlaybackSubscription, PlaylistSubscription);
