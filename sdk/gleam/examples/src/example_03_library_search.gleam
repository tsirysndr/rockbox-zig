//// 03 — Library search
////
//// Edit `search_term` below to query something different.
////
////   gleam run -m example_03_library_search

import gleam/int
import gleam/io
import gleam/list
import helper
import rockbox/library

const search_term = "love"

pub fn main() {
  let client = helper.client()

  let assert Ok(results) = library.search(client, search_term)

  io.println("Search: \"" <> search_term <> "\"")

  io.println(
    "  Artists  (" <> int.to_string(list.length(results.artists)) <> "):",
  )
  results.artists
  |> list.take(5)
  |> list.each(fn(a) { io.println("    • " <> a.name) })

  io.println(
    "  Albums   (" <> int.to_string(list.length(results.albums)) <> "):",
  )
  results.albums
  |> list.take(5)
  |> list.each(fn(a) {
    io.println(
      "    • " <> a.title <> " — " <> a.artist <> " (" <> int.to_string(a.year) <> ")",
    )
  })

  io.println(
    "  Tracks   (" <> int.to_string(list.length(results.tracks)) <> "):",
  )
  results.tracks
  |> list.take(5)
  |> list.each(fn(t) { io.println("    • " <> t.title <> " — " <> t.artist) })
}
