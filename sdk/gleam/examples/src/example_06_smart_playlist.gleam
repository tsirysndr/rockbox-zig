//// 06 — Smart playlist with the Rules builder
////
////   gleam run -m example_06_smart_playlist

import gleam/int
import gleam/io
import gleam/list
import gleam/option.{Some}
import helper
import rockbox/smart_playlists
import rockbox/smart_playlists/rules

pub fn main() {
  let client = helper.client()

  let r =
    rules.all_of()
    |> rules.where("play_count", rules.Gte, rules.int(1))
    |> rules.sort("play_count", rules.Desc)
    |> rules.limit(25)

  let input =
    smart_playlists.new("Most played (demo)", rules.to_string(r))
    |> smart_playlists.with_description("Top 25 most-played tracks")

  let assert Ok(sp) = smart_playlists.create(client, input)
  io.println("Created smart playlist: " <> sp.id)

  let assert Ok(ids) = smart_playlists.track_ids(client, sp.id)
  io.println(
    "Currently resolves to " <> int.to_string(list.length(ids)) <> " tracks",
  )

  case list.first(ids) {
    Ok(top_id) ->
      case smart_playlists.track_stats(client, top_id) {
        Ok(Some(stats)) ->
          io.println(
            "Top track stats: played "
            <> int.to_string(stats.play_count)
            <> "× (skipped "
            <> int.to_string(stats.skip_count)
            <> "×)",
          )
        _ -> Nil
      }
    Error(_) -> Nil
  }

  let _ = smart_playlists.delete(client, sp.id)
  io.println("Cleaned up demo smart playlist.")
}
