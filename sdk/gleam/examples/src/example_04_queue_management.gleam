//// 04 — Queue management
////
//// Inspect the live playback queue. Mutating helpers (`insert_tracks`,
//// `remove_track`, `clear`, `shuffle`) are commented out at the bottom.
////
////   gleam run -m example_04_queue_management

import gleam/int
import gleam/io
import gleam/list
import helper
import rockbox/playlist

pub fn main() {
  let client = helper.client()

  let assert Ok(queue) = playlist.current(client)
  io.println(
    "Queue: "
    <> int.to_string(queue.amount)
    <> " tracks, currently at index "
    <> int.to_string(queue.index),
  )

  queue.tracks
  |> list.take(10)
  |> list.index_map(fn(t, i) { #(t, i) })
  |> list.each(fn(pair) {
    let #(t, i) = pair
    let marker = case i == queue.index {
      True -> "▶"
      False -> " "
    }
    io.println(
      marker
      <> " "
      <> int.to_string(i + 1)
      <> ". "
      <> t.title
      <> " — "
      <> t.artist,
    )
  })

  // Pipe-friendly chained ops:
  //
  //   let _ = playlist.clear(client)
  //   let _ = playlist.insert_tracks(
  //     client, ["/Music/a.mp3", "/Music/b.mp3"], types.Last, option.None,
  //   )
  //   let _ = playlist.shuffle(client)
}
