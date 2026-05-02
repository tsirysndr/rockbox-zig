//// 08 — EQ configuration
////
//// Enable the equalizer with a 5-band bass-boost / presence-cut profile.
////
////   gleam run -m example_08_eq_config

import gleam/bool
import gleam/int
import gleam/io
import gleam/list
import helper
import rockbox/settings
import rockbox/types.{EqBandSetting}

pub fn main() {
  let client = helper.client()

  let bands = [
    EqBandSetting(cutoff: 60, q: 7, gain: 3),
    EqBandSetting(cutoff: 200, q: 7, gain: 0),
    EqBandSetting(cutoff: 800, q: 7, gain: 0),
    EqBandSetting(cutoff: 4000, q: 7, gain: -2),
    EqBandSetting(cutoff: 12_000, q: 7, gain: 1),
  ]

  let patch =
    settings.patch()
    |> settings.set_eq_enabled(True)
    |> settings.set_eq_precut(-3)
    |> settings.set_eq_bands(bands)

  let assert Ok(_) = settings.save(client, patch)

  let assert Ok(s) = settings.get(client)
  io.println("EQ enabled: " <> bool.to_string(s.eq_enabled))
  io.println("EQ precut:  " <> int.to_string(s.eq_precut))

  list.each(s.eq_band_settings, fn(b) {
    io.println(
      "  "
      <> helper.pad_int(b.cutoff, 6)
      <> " Hz  q="
      <> int.to_string(b.q)
      <> "  "
      <> int.to_string(b.gain)
      <> " dB",
    )
  })
}
