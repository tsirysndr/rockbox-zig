//// The current/active playback queue (Mopidy-style "tracklist").
////
//// For named, persisted playlists see `rockbox/saved_playlists`.

import gleam/dynamic/decode
import gleam/json
import gleam/option.{type Option, None, Some}
import rockbox.{type Client}
import rockbox/error.{type Error}
import rockbox/internal/transport
import rockbox/types.{type InsertPosition, type Playlist}

const track_fields = "
  fragment TrackFields on Track {
    id title artist album genre disc trackString yearString
    composer comment albumArtist grouping
    discnum tracknum layer year bitrate frequency
    filesize length elapsed path
    albumId artistId genreId albumArt
  }
"

/// Snapshot of the active queue.
pub fn current(client: Client) -> Result(Playlist, Error) {
  let decoder = {
    use playlist <- decode.field(
      "playlistGetCurrent",
      types.playlist_decoder(),
    )
    decode.success(playlist)
  }
  let q = track_fields <> "
    query CurrentPlaylist {
      playlistGetCurrent {
        amount index maxPlaylistSize firstIndex
        lastInsertPos seed lastShuffledStart
        tracks { ...TrackFields }
      }
    }
  "
  rockbox.query(client, q, json.object([]), decoder)
}

/// Number of tracks currently queued.
pub fn amount(client: Client) -> Result(Int, Error) {
  let decoder = {
    use n <- decode.field("playlistAmount", decode.int)
    decode.success(n)
  }
  rockbox.query(
    client,
    "query PlaylistAmount { playlistAmount }",
    json.object([]),
    decoder,
  )
}

// ---------------------------------------------------------------------------
// Queue mutations
// ---------------------------------------------------------------------------

/// Insert a list of paths or track IDs into the queue at the given position.
///
/// Pass `option.None` for `playlist_id` to target the active queue.
pub fn insert_tracks(
  client: Client,
  paths: List(String),
  position: InsertPosition,
  playlist_id: Option(String),
) -> Result(Nil, Error) {
  let vars =
    transport.variables([
      #("playlistId", option.map(playlist_id, json.string)),
      #("position", Some(json.int(types.insert_position_to_int(position)))),
      #(
        "tracks",
        Some(json.array(paths, json.string)),
      ),
    ])
  rockbox.execute(
    client,
    "mutation InsertTracks($playlistId: String, $position: Int!, $tracks: [String!]!) {
       insertTracks(playlistId: $playlistId, position: $position, tracks: $tracks)
     }",
    vars,
  )
}

/// Append every track in a directory (optionally recursive) to the queue.
pub fn insert_directory(
  client: Client,
  directory: String,
  position: InsertPosition,
  playlist_id: Option(String),
) -> Result(Nil, Error) {
  let vars =
    transport.variables([
      #("playlistId", option.map(playlist_id, json.string)),
      #("position", Some(json.int(types.insert_position_to_int(position)))),
      #("directory", Some(json.string(directory))),
    ])
  rockbox.execute(
    client,
    "mutation InsertDirectory($playlistId: String, $position: Int!, $directory: String!) {
       insertDirectory(playlistId: $playlistId, position: $position, directory: $directory)
     }",
    vars,
  )
}

/// Append every track on an album to the queue.
pub fn insert_album(
  client: Client,
  album_id: String,
  position: InsertPosition,
) -> Result(Nil, Error) {
  rockbox.execute(
    client,
    "mutation InsertAlbum($albumId: String!, $position: Int!) {
       insertAlbum(albumId: $albumId, position: $position)
     }",
    json.object([
      #("albumId", json.string(album_id)),
      #("position", json.int(types.insert_position_to_int(position))),
    ]),
  )
}

pub fn remove_track(client: Client, index: Int) -> Result(Nil, Error) {
  rockbox.execute(
    client,
    "mutation RemoveTrack($index: Int!) { playlistRemoveTrack(index: $index) }",
    json.object([#("index", json.int(index))]),
  )
}

/// Empty the active queue.
pub fn clear(client: Client) -> Result(Nil, Error) {
  rockbox.execute(
    client,
    "mutation ClearPlaylist { playlistRemoveAllTracks }",
    json.object([]),
  )
}

/// Shuffle the active queue in place.
pub fn shuffle(client: Client) -> Result(Nil, Error) {
  rockbox.execute(
    client,
    "mutation ShufflePlaylist { shufflePlaylist }",
    json.object([]),
  )
}

/// Replace the queue with a new ad-hoc playlist and start playing it.
pub fn create(
  client: Client,
  name: String,
  tracks: List(String),
) -> Result(Nil, Error) {
  rockbox.execute(
    client,
    "mutation CreatePlaylist($name: String!, $tracks: [String!]!) {
       playlistCreate(name: $name, tracks: $tracks)
     }",
    json.object([
      #("name", json.string(name)),
      #("tracks", json.array(tracks, json.string)),
    ]),
  )
}

/// Optional knobs for `start`.
pub opaque type StartOptions {
  StartOptions(
    start_index: Option(Int),
    elapsed: Option(Int),
    offset: Option(Int),
  )
}

pub fn start_options() -> StartOptions {
  StartOptions(start_index: None, elapsed: None, offset: None)
}

pub fn at_index(opts: StartOptions, value: Int) -> StartOptions {
  StartOptions(..opts, start_index: Some(value))
}

pub fn at_elapsed(opts: StartOptions, value: Int) -> StartOptions {
  StartOptions(..opts, elapsed: Some(value))
}

pub fn at_offset(opts: StartOptions, value: Int) -> StartOptions {
  StartOptions(..opts, offset: Some(value))
}

/// Start playing the active queue from the given position.
pub fn start(client: Client, options: StartOptions) -> Result(Nil, Error) {
  let pairs = [
    #("startIndex", option.map(options.start_index, json.int)),
    #("elapsed", option.map(options.elapsed, json.int)),
    #("offset", option.map(options.offset, json.int)),
  ]
  rockbox.execute(
    client,
    "mutation PlaylistStart($startIndex: Int, $elapsed: Int, $offset: Int) {
       playlistStart(startIndex: $startIndex, elapsed: $elapsed, offset: $offset)
     }",
    transport.variables(pairs),
  )
}

/// Resume an interrupted queue from the saved position.
pub fn resume(client: Client) -> Result(Nil, Error) {
  rockbox.execute(
    client,
    "mutation PlaylistResume { playlistResume }",
    json.object([]),
  )
}

