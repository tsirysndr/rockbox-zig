import gleam/option
import gleeunit
import rockbox
import rockbox/types

pub fn main() -> Nil {
  gleeunit.main()
}

pub fn default_url_test() {
  let client = rockbox.new() |> rockbox.connect
  assert rockbox.http_url(client) == "http://localhost:6062/graphql"
}

pub fn host_override_test() {
  let client =
    rockbox.new()
    |> rockbox.host("rockbox.local")
    |> rockbox.port(8080)
    |> rockbox.connect

  assert rockbox.http_url(client) == "http://rockbox.local:8080/graphql"
}

pub fn url_override_test() {
  let client =
    rockbox.new()
    |> rockbox.host("ignored")
    |> rockbox.url("https://api.example.com/graphql")
    |> rockbox.connect

  assert rockbox.http_url(client) == "https://api.example.com/graphql"
}

pub fn at_helper_test() {
  let client = rockbox.at(host: "192.168.1.10", port: 6062)
  assert rockbox.http_url(client) == "http://192.168.1.10:6062/graphql"
}

pub fn playback_status_round_trip_test() {
  assert types.playback_status_from_int(0) == types.Stopped
  assert types.playback_status_from_int(1) == types.Playing
  assert types.playback_status_from_int(3) == types.Paused

  assert types.playback_status_to_int(types.Stopped) == 0
  assert types.playback_status_to_int(types.Playing) == 1
  assert types.playback_status_to_int(types.Paused) == 3
}

pub fn insert_position_test() {
  assert types.insert_position_to_int(types.Next) == 0
  assert types.insert_position_to_int(types.AfterCurrent) == 1
  assert types.insert_position_to_int(types.Last) == 2
  assert types.insert_position_to_int(types.First) == 3
}

pub fn is_directory_test() {
  let dir =
    types.Entry(
      name: "Music",
      attr: 0x10,
      time_write: 0,
      customaction: 0,
      display_name: option.None,
    )
  let file =
    types.Entry(
      name: "song.mp3",
      attr: 0x00,
      time_write: 0,
      customaction: 0,
      display_name: option.None,
    )

  assert types.is_directory(dir)
  assert !types.is_directory(file)
}
