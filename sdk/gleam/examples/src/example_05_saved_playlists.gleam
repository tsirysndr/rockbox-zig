//// 05 — Saved playlists
////
////   gleam run -m example_05_saved_playlists

import gleam/int
import gleam/io
import gleam/list
import gleam/option.{None}
import helper
import rockbox/saved_playlists

pub fn main() {
  let client = helper.client()

  let assert Ok(lists) = saved_playlists.list(client, None)
  io.println(
    "You have " <> int.to_string(list.length(lists)) <> " saved playlist(s):",
  )

  list.each(lists, fn(pl) {
    io.println(
      "  • "
      <> pl.name
      <> "  —  "
      <> int.to_string(pl.track_count)
      <> " tracks  (id: "
      <> pl.id
      <> ")",
    )
  })

  // Create + add tracks + delete (uncomment to demo):
  //
  //   let input =
  //     saved_playlists.new("Demo")
  //     |> saved_playlists.with_description("test")
  //   let assert Ok(pl) = saved_playlists.create(client, input)
  //   let _ =
  //     saved_playlists.add_tracks(client, pl.id, ["track-id-1", "track-id-2"])
  //   let _ = saved_playlists.delete(client, pl.id)
}
