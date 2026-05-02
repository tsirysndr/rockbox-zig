//// 02 — Now Playing (polling)
////
//// The Gleam SDK is request/response only — there's no WebSocket subscription
//// helper yet. This example polls `currentTrack` and prints when the playing
//// track changes. Press Ctrl+C to exit.
////
////   gleam run -m example_02_now_playing

import gleam/erlang/process
import gleam/io
import gleam/option.{type Option, None, Some}
import helper
import rockbox.{type Client}
import rockbox/playback
import rockbox/types.{type Track}

const poll_interval_ms = 1000

pub fn main() {
  let client = helper.client()
  io.println("Watching for track changes (Ctrl+C to exit).")
  loop(client, None)
}

fn loop(client: Client, last_seen: Option(String)) {
  let now = case playback.current_track(client) {
    Ok(track) -> track
    Error(_) -> None
  }

  let last_seen = case now, last_seen {
    Some(track), previous -> {
      let key = track_key(track)
      case Some(key) == previous {
        True -> previous
        False -> {
          io.println(
            "▶ "
            <> track.title
            <> " — "
            <> track.artist
            <> "  ("
            <> helper.fmt_ms(track.length)
            <> ")",
          )
          Some(key)
        }
      }
    }
    None, Some(_) -> {
      io.println("· no track")
      None
    }
    None, None -> None
  }

  process.sleep(poll_interval_ms)
  loop(client, last_seen)
}

fn track_key(track: Track) -> String {
  case track.id {
    Some(id) -> id
    None -> track.path
  }
}
