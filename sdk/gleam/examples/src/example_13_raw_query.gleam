//// 13 — Raw GraphQL escape hatch
////
//// For operations the SDK doesn't expose directly, drop down to
//// `rockbox.query/4` and supply your own decoder.
////
////   gleam run -m example_13_raw_query

import gleam/dynamic/decode
import gleam/int
import gleam/io
import gleam/json
import gleam/option.{Some}
import helper
import rockbox

pub fn main() {
  let client = helper.client()

  let version_decoder = {
    use v <- decode.field("rockboxVersion", decode.string)
    decode.success(v)
  }

  let assert Ok(version) =
    rockbox.query(
      client,
      "query Version { rockboxVersion }",
      json.object([]),
      version_decoder,
    )
  io.println("rockboxd " <> version)

  let album_decoder = {
    use album <- decode.field(
      "album",
      decode.optional({
        use id <- decode.field("id", decode.string)
        use title <- decode.optional_field("title", "", decode.string)
        use artist <- decode.optional_field("artist", "", decode.string)
        use year <- decode.optional_field("year", 0, decode.int)
        decode.success(#(id, title, artist, year))
      }),
    )
    decode.success(album)
  }

  case
    rockbox.query(
      client,
      "query Album($id: String!) { album(id: $id) { id title artist year } }",
      json.object([#("id", json.string("demo-id-or-use-a-real-one"))]),
      album_decoder,
    )
  {
    Ok(Some(#(id, title, artist, year))) ->
      io.println(
        "album: id="
        <> id
        <> " title="
        <> title
        <> " artist="
        <> artist
        <> " year="
        <> int.to_string(year),
      )
    Ok(_) -> io.println("album: (not found)")
    Error(_) -> io.println("album: query failed")
  }
}
