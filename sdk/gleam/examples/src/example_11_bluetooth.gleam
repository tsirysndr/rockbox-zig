//// 11 — Bluetooth (Linux only)
////
//// Lists paired devices. Uncomment one of the blocks at the bottom to scan
//// or connect/disconnect a specific address.
////
////   gleam run -m example_11_bluetooth

import gleam/int
import gleam/io
import gleam/list
import gleam/option.{None, Some}
import gleam/string
import helper
import rockbox/bluetooth
import rockbox/types.{type BluetoothDevice}

pub fn main() {
  let client = helper.client()

  let assert Ok(devices) = bluetooth.devices(client)
  io.println(
    "Paired devices (" <> int.to_string(list.length(devices)) <> "):",
  )
  list.each(devices, fn(d) { io.println(format_device(d)) })

  // Scan for 10 seconds:
  //
  //   let assert Ok(found) = bluetooth.scan(client, Some(10))
  //   list.each(found, fn(d) { io.println(format_device(d)) })

  // Connect / disconnect a specific address:
  //
  //   let _ = bluetooth.connect(client, "AA:BB:CC:DD:EE:FF")
  //   let _ = bluetooth.disconnect(client, "AA:BB:CC:DD:EE:FF")
}

fn format_device(d: BluetoothDevice) -> String {
  let flags =
    [
      flag(d.connected, "connected"),
      flag(d.paired, "paired"),
      flag(d.trusted, "trusted"),
    ]
    |> list.filter_map(fn(x) { x })
    |> string.join(", ")

  let rssi = case d.rssi {
    Some(v) -> " " <> int.to_string(v) <> " dBm"
    None -> ""
  }

  "  "
  <> d.address
  <> "  "
  <> string.pad_end(d.name, 28, " ")
  <> rssi
  <> "  ["
  <> flags
  <> "]"
}

fn flag(condition: Bool, label: String) -> Result(String, Nil) {
  case condition {
    True -> Ok(label)
    False -> Error(Nil)
  }
}
