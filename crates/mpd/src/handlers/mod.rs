pub mod batch;
pub mod browse;
pub mod library;
pub mod playback;
pub mod queue;
pub mod system;

#[derive(Debug, Clone)]
pub enum Subsystem {
    Database,
    Update,
    StoredPlaylist,
    Playlist,
    Player,
    Mixer,
    Output,
    Options,
    Partition,
    Sticker,
    Subscription,
    Message,
    Neighbor,
    Mount,
}

impl ToString for Subsystem {
    fn to_string(&self) -> String {
        match self {
            Subsystem::Database => "database".to_string(),
            Subsystem::Update => "update".to_string(),
            Subsystem::StoredPlaylist => "stored_playlist".to_string(),
            Subsystem::Playlist => "playlist".to_string(),
            Subsystem::Player => "player".to_string(),
            Subsystem::Mixer => "mixer".to_string(),
            Subsystem::Output => "output".to_string(),
            Subsystem::Options => "options".to_string(),
            Subsystem::Partition => "partition".to_string(),
            Subsystem::Sticker => "sticker".to_string(),
            Subsystem::Subscription => "subscription".to_string(),
            Subsystem::Message => "message".to_string(),
            Subsystem::Neighbor => "neighbor".to_string(),
            Subsystem::Mount => "mount".to_string(),
        }
    }
}
