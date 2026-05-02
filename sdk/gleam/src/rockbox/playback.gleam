//// Transport controls and "play this thing" shortcuts.
////
//// ```gleam
//// import rockbox
//// import rockbox/playback
////
//// let client = rockbox.connect(rockbox.new())
////
//// let _ = playback.play_track(client, "/Music/Miles Davis/Kind of Blue/01.flac")
//// let _ = playback.pause(client)
//// let _ = playback.next(client)
//// ```

import gleam/dynamic/decode
import gleam/json
import gleam/option.{type Option, None, Some}
import rockbox.{type Client}
import rockbox/error.{type Error}
import rockbox/internal/transport
import rockbox/types.{type PlaybackStatus, type Track}

const track_fields = "
  fragment TrackFields on Track {
    id title artist album genre disc trackString yearString
    composer comment albumArtist grouping
    discnum tracknum layer year bitrate frequency
    filesize length elapsed path
    albumId artistId genreId albumArt
  }
"

// ---------------------------------------------------------------------------
// Status & current track
// ---------------------------------------------------------------------------

/// Raw numeric playback status as the firmware reports it.
pub fn raw_status(client: Client) -> Result(Int, Error) {
  let decoder = {
    use status <- decode.field("status", decode.int)
    decode.success(status)
  }
  rockbox.query(client, "query PlaybackStatus { status }", json.object([]), decoder)
}

/// Typed playback status (`Stopped`, `Playing`, `Paused`, …).
pub fn status(client: Client) -> Result(PlaybackStatus, Error) {
  case raw_status(client) {
    Ok(value) -> Ok(types.playback_status_from_int(value))
    Error(err) -> Error(err)
  }
}

/// The track currently loaded for playback (may not be playing). Returns
/// `Ok(None)` if no track is queued.
pub fn current_track(client: Client) -> Result(Option(Track), Error) {
  let decoder = {
    use track <- decode.field(
      "currentTrack",
      decode.optional(types.track_decoder()),
    )
    decode.success(track)
  }
  let q = track_fields <> "
    query CurrentTrack { currentTrack { ...TrackFields } }
  "
  rockbox.query(client, q, json.object([]), decoder)
}

/// The track that will play after the current one finishes. `Ok(None)` if the
/// queue ends with the current track.
pub fn next_track(client: Client) -> Result(Option(Track), Error) {
  let decoder = {
    use track <- decode.field(
      "nextTrack",
      decode.optional(types.track_decoder()),
    )
    decode.success(track)
  }
  let q = track_fields <> "
    query NextTrack { nextTrack { ...TrackFields } }
  "
  rockbox.query(client, q, json.object([]), decoder)
}

/// Position of the audio file the codec is currently reading from, in bytes.
pub fn file_position(client: Client) -> Result(Int, Error) {
  let decoder = {
    use pos <- decode.field("getFilePosition", decode.int)
    decode.success(pos)
  }
  rockbox.query(
    client,
    "query FilePosition { getFilePosition }",
    json.object([]),
    decoder,
  )
}

// ---------------------------------------------------------------------------
// Transport controls
// ---------------------------------------------------------------------------

/// Start playback at `elapsed` ms with the codec offset set to `offset` bytes.
/// Pass `0, 0` to start from the beginning.
pub fn play(client: Client, elapsed: Int, offset: Int) -> Result(Nil, Error) {
  rockbox.execute(
    client,
    "mutation Play($elapsed: Long!, $offset: Long!) { play(elapsed: $elapsed, offset: $offset) }",
    json.object([
      #("elapsed", json.int(elapsed)),
      #("offset", json.int(offset)),
    ]),
  )
}

/// Pause playback.
pub fn pause(client: Client) -> Result(Nil, Error) {
  rockbox.execute(client, "mutation Pause { pause }", json.object([]))
}

/// Resume playback after a pause.
pub fn resume(client: Client) -> Result(Nil, Error) {
  rockbox.execute(client, "mutation Resume { resume }", json.object([]))
}

/// Skip to the next track in the queue.
pub fn next(client: Client) -> Result(Nil, Error) {
  rockbox.execute(client, "mutation Next { next }", json.object([]))
}

/// Go back to the previous track.
pub fn previous(client: Client) -> Result(Nil, Error) {
  rockbox.execute(client, "mutation Previous { previous }", json.object([]))
}

/// Seek the current track to an absolute position in milliseconds.
pub fn seek(client: Client, position_ms: Int) -> Result(Nil, Error) {
  rockbox.execute(
    client,
    "mutation Seek($newTime: Int!) { fastForwardRewind(newTime: $newTime) }",
    json.object([#("newTime", json.int(position_ms))]),
  )
}

/// Stop playback and tear down the audio engine.
pub fn stop(client: Client) -> Result(Nil, Error) {
  rockbox.execute(client, "mutation Stop { hardStop }", json.object([]))
}

/// Flush the codec buffer and reload the queue from disk.
pub fn flush_and_reload(client: Client) -> Result(Nil, Error) {
  rockbox.execute(
    client,
    "mutation FlushReload { flushAndReloadTracks }",
    json.object([]),
  )
}

// ---------------------------------------------------------------------------
// Play helpers — single-call shortcuts
// ---------------------------------------------------------------------------

/// Play a single file by absolute path.
pub fn play_track(client: Client, path: String) -> Result(Nil, Error) {
  rockbox.execute(
    client,
    "mutation PlayTrack($path: String!) { playTrack(path: $path) }",
    json.object([#("path", json.string(path))]),
  )
}

/// Optional knobs accepted by every `play_*` shortcut.
///
/// Build one with `play_options()` and chain `with_shuffle` / `with_position`:
///
/// ```gleam
/// let opts =
///   playback.play_options()
///   |> playback.with_shuffle(True)
///   |> playback.with_position(2)
///
/// let _ = playback.play_album(client, "abc-123", opts)
/// ```
pub opaque type PlayOptions {
  PlayOptions(shuffle: Option(Bool), position: Option(Int))
}

/// Default play options — no shuffle, append at the end.
pub fn play_options() -> PlayOptions {
  PlayOptions(shuffle: None, position: None)
}

/// Toggle shuffle for the resulting queue.
pub fn with_shuffle(opts: PlayOptions, value: Bool) -> PlayOptions {
  PlayOptions(..opts, shuffle: Some(value))
}

/// Set the queue position to start playback at.
pub fn with_position(opts: PlayOptions, value: Int) -> PlayOptions {
  PlayOptions(..opts, position: Some(value))
}

fn play_options_pairs(
  opts: PlayOptions,
) -> List(#(String, Option(json.Json))) {
  [
    #("shuffle", option.map(opts.shuffle, json.bool)),
    #("position", option.map(opts.position, json.int)),
  ]
}

/// Replace the queue with every track on an album and start playing.
pub fn play_album(
  client: Client,
  album_id: String,
  options: PlayOptions,
) -> Result(Nil, Error) {
  let vars =
    transport.variables([
      #("albumId", Some(json.string(album_id))),
      ..play_options_pairs(options)
    ])
  rockbox.execute(
    client,
    "mutation PlayAlbum($albumId: String!, $shuffle: Boolean, $position: Int) {
       playAlbum(albumId: $albumId, shuffle: $shuffle, position: $position)
     }",
    vars,
  )
}

/// Replace the queue with every track an artist has and start playing.
pub fn play_artist(
  client: Client,
  artist_id: String,
  options: PlayOptions,
) -> Result(Nil, Error) {
  let vars =
    transport.variables([
      #("artistId", Some(json.string(artist_id))),
      ..play_options_pairs(options)
    ])
  rockbox.execute(
    client,
    "mutation PlayArtist($artistId: String!, $shuffle: Boolean, $position: Int) {
       playArtistTracks(artistId: $artistId, shuffle: $shuffle, position: $position)
     }",
    vars,
  )
}

/// Play a saved playlist by ID.
pub fn play_playlist(
  client: Client,
  playlist_id: String,
  options: PlayOptions,
) -> Result(Nil, Error) {
  let vars =
    transport.variables([
      #("playlistId", Some(json.string(playlist_id))),
      ..play_options_pairs(options)
    ])
  rockbox.execute(
    client,
    "mutation PlayPlaylist($playlistId: String!, $shuffle: Boolean, $position: Int) {
       playPlaylist(playlistId: $playlistId, shuffle: $shuffle, position: $position)
     }",
    vars,
  )
}

/// Queue and play every audio file in a directory.
pub fn play_directory(
  client: Client,
  path: String,
  recurse: Bool,
  options: PlayOptions,
) -> Result(Nil, Error) {
  let vars =
    transport.variables([
      #("path", Some(json.string(path))),
      #("recurse", Some(json.bool(recurse))),
      ..play_options_pairs(options)
    ])
  rockbox.execute(
    client,
    "mutation PlayDirectory($path: String!, $recurse: Boolean, $shuffle: Boolean, $position: Int) {
       playDirectory(path: $path, recurse: $recurse, shuffle: $shuffle, position: $position)
     }",
    vars,
  )
}

/// Play every liked track.
pub fn play_liked_tracks(
  client: Client,
  options: PlayOptions,
) -> Result(Nil, Error) {
  let vars = transport.variables(play_options_pairs(options))
  rockbox.execute(
    client,
    "mutation PlayLikedTracks($shuffle: Boolean, $position: Int) {
       playLikedTracks(shuffle: $shuffle, position: $position)
     }",
    vars,
  )
}

/// Play the entire library.
pub fn play_all_tracks(
  client: Client,
  options: PlayOptions,
) -> Result(Nil, Error) {
  let vars = transport.variables(play_options_pairs(options))
  rockbox.execute(
    client,
    "mutation PlayAllTracks($shuffle: Boolean, $position: Int) {
       playAllTracks(shuffle: $shuffle, position: $position)
     }",
    vars,
  )
}
