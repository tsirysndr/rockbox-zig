//// File-tree browsing for both the local filesystem and UPnP servers.

import gleam/dynamic/decode
import gleam/json
import gleam/list
import gleam/option.{type Option}
import rockbox.{type Client}
import rockbox/error.{type Error}
import rockbox/internal/transport
import rockbox/types.{type Entry}

/// Both directories and files at `path`. Pass `option.None` to read the root.
pub fn entries(
  client: Client,
  path: Option(String),
) -> Result(List(Entry), Error) {
  let decoder = {
    use entries <- decode.field(
      "treeGetEntries",
      decode.list(types.entry_decoder()),
    )
    decode.success(entries)
  }
  let vars = transport.variables([#("path", option.map(path, json.string))])
  rockbox.query(
    client,
    "query Browse($path: String) {
       treeGetEntries(path: $path) { name attr timeWrite customaction displayName }
     }",
    vars,
    decoder,
  )
}

/// Subdirectories only.
pub fn directories(
  client: Client,
  path: Option(String),
) -> Result(List(Entry), Error) {
  case entries(client, path) {
    Ok(all) -> Ok(list.filter(all, types.is_directory))
    Error(err) -> Error(err)
  }
}

/// Files only (everything that isn't a directory).
pub fn files(
  client: Client,
  path: Option(String),
) -> Result(List(Entry), Error) {
  case entries(client, path) {
    Ok(all) ->
      Ok(list.filter(all, fn(e) { !types.is_directory(e) }))
    Error(err) -> Error(err)
  }
}
