//// 01 — Basic playback
////
//// Inspect the current track and toggle play/pause based on status.
//// Idempotent: run twice and it flips between Playing and Paused.
////
////   gleam run -m example_01_basic_playback

import gleam/int
import gleam/io
import gleam/option.{None, Some}
import helper
import rockbox/playback
import rockbox/types

pub fn main() {
  let client = helper.client()

  let assert Ok(status) = playback.status(client)
  io.println("Status: " <> status_label(status))

  case playback.current_track(client) {
    Ok(Some(track)) -> {
      let pct = case track.length {
        0 -> 0
        len -> track.elapsed * 100 / len
      }
      io.println("Now: " <> track.title <> " — " <> track.artist)
      io.println(
        "     "
        <> helper.fmt_ms(track.elapsed)
        <> " / "
        <> helper.fmt_ms(track.length)
        <> " ("
        <> int.to_string(pct)
        <> "%)",
      )
    }
    Ok(None) -> io.println("Nothing is playing.")
    Error(_) -> io.println("Could not read current track.")
  }

  case status {
    types.Playing -> {
      let _ = playback.pause(client)
      io.println("→ paused")
    }
    types.Paused -> {
      let _ = playback.resume(client)
      io.println("→ resumed")
    }
    _ -> Nil
  }
}

fn status_label(status: types.PlaybackStatus) -> String {
  case status {
    types.Stopped -> "stopped"
    types.Playing -> "playing"
    types.Paused -> "paused"
    types.UnknownStatus(n) -> "unknown(" <> int.to_string(n) <> ")"
  }
}
