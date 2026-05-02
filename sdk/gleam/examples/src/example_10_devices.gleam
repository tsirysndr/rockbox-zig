//// 10 — List remote output devices (Chromecast / AirPlay / …)
////
////   gleam run -m example_10_devices

import gleam/int
import gleam/io
import gleam/list
import helper
import rockbox/devices
import rockbox/types.{type Device}

pub fn main() {
  let client = helper.client()
  let assert Ok(found) = devices.list(client)

  io.println(
    "Discovered " <> int.to_string(list.length(found)) <> " device(s):",
  )

  list.each(found, fn(d) {
    let status = case d.is_connected {
      True -> "● connected"
      False -> "○ available"
    }
    io.println(
      "  ["
      <> device_kind(d)
      <> "] "
      <> d.name
      <> " ("
      <> d.ip
      <> ":"
      <> int.to_string(d.port)
      <> ")  —  "
      <> status,
    )
  })
}

fn device_kind(d: Device) -> String {
  case d.is_cast_device, d.is_source_device {
    True, _ -> "Cast"
    False, True -> "Source"
    False, False -> "Other"
  }
}
