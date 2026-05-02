//// Volume control.

import gleam/dynamic/decode
import gleam/json
import rockbox.{type Client}
import rockbox/error.{type Error}
import rockbox/types.{type VolumeInfo}

/// Current volume with the firmware-reported `min` / `max` range.
pub fn get_volume(client: Client) -> Result(VolumeInfo, Error) {
  let decoder = {
    use volume <- decode.field("volume", types.volume_info_decoder())
    decode.success(volume)
  }
  rockbox.query(
    client,
    "query Volume { volume { volume min max } }",
    json.object([]),
    decoder,
  )
}

/// Adjust volume by a relative number of steps. Returns the new absolute
/// volume.
pub fn adjust_volume(client: Client, steps: Int) -> Result(Int, Error) {
  let decoder = {
    use volume <- decode.field("adjustVolume", decode.int)
    decode.success(volume)
  }
  rockbox.query(
    client,
    "mutation AdjustVolume($steps: Int!) { adjustVolume(steps: $steps) }",
    json.object([#("steps", json.int(steps))]),
    decoder,
  )
}

/// Bump the volume up by one step.
pub fn volume_up(client: Client) -> Result(Int, Error) {
  adjust_volume(client, 1)
}

/// Drop the volume down by one step.
pub fn volume_down(client: Client) -> Result(Int, Error) {
  adjust_volume(client, -1)
}
