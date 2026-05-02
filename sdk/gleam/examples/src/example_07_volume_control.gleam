//// 07 — Volume control
////
////   gleam run -m example_07_volume_control

import gleam/int
import gleam/io
import helper
import rockbox/sound

pub fn main() {
  let client = helper.client()

  let assert Ok(vol) = sound.get_volume(client)
  io.println(
    "Current: "
    <> int.to_string(vol.volume)
    <> " (range "
    <> int.to_string(vol.min)
    <> ".."
    <> int.to_string(vol.max)
    <> ")",
  )

  let assert Ok(after_up) = sound.volume_up(client)
  io.println("After +1: " <> int.to_string(after_up))

  let assert Ok(_) = sound.volume_down(client)
  io.println("Reverted.")
}
