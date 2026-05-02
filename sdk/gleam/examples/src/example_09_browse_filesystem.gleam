//// 09 — Browse the filesystem
////
//// Set `browse_path` to `Some("/Music/Pink Floyd")` to walk a sub-directory,
//// or leave it as `None` to list the configured `music_dir` root.
////
////   gleam run -m example_09_browse_filesystem

import gleam/io
import gleam/list
import gleam/option.{type Option, None}
import helper
import rockbox/browse
import rockbox/types

const browse_path: Option(String) = None

pub fn main() {
  let client = helper.client()

  let assert Ok(entries) = browse.entries(client, browse_path)

  list.each(entries, fn(e) {
    let icon = case types.is_directory(e) {
      True -> "[dir] "
      False -> "      "
    }
    io.println(icon <> e.name)
  })
}
