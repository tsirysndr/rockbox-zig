//// 12 — Sleep timer
////
//// Polling-based version of the Elixir plugin example: stops playback after
//// `minutes` minutes, but bails out early if playback was stopped manually.
////
//// The Gleam SDK has no plugin/event-bus yet, so this is a straight loop —
//// short-circuiting via early return when status flips to Stopped.
////
////   gleam run -m example_12_sleep_timer

import gleam/erlang/process
import gleam/int
import gleam/io
import helper
import rockbox.{type Client}
import rockbox/playback
import rockbox/types

const minutes: Int = 30

const tick_ms: Int = 1000

pub fn main() {
  let client = helper.client()
  let total_ticks = minutes * 60

  io.println(
    "💤 Sleep timer armed — stopping playback in "
    <> int.to_string(minutes)
    <> " minute(s).",
  )

  loop(client, total_ticks)
}

fn loop(client: Client, ticks_remaining: Int) {
  case ticks_remaining {
    n if n <= 0 -> {
      io.println("💤 Time's up — stopping playback.")
      let _ = playback.stop(client)
      Nil
    }
    _ ->
      case playback.status(client) {
        Ok(types.Stopped) -> {
          io.println("💤 Playback stopped manually — sleep timer cancelled.")
          Nil
        }
        _ -> {
          process.sleep(tick_ms)
          loop(client, ticks_remaining - 1)
        }
      }
  }
}
