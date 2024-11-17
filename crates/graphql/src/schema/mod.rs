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

#[macro_export]
macro_rules! check_and_load_player {
    ($client:expr, $tracks:expr, $shuffle:expr) => {
        let response = $client
            .get(&format!("{}/player", rockbox_url()))
            .send()
            .await?;
        let player = response.json::<Device>().await?;

        // connected to a player
        if !player.host.is_empty() && player.port != 0 {
            let client = reqwest::Client::new();
            let body = serde_json::json!({
                "tracks": $tracks,
                "shuffle": $shuffle,
            });

            client
                .put(&format!("{}/player/load", rockbox_url()))
                .json(&body)
                .send()
                .await?;
            return Ok(0);
        }
    };
}
