//// Shared helpers for the example modules.
////
//// Edit `client()` to point at a different host/port if your `rockboxd` is
//// not running on `localhost:6062`.

import gleam/int
import gleam/string
import rockbox.{type Client}

pub fn client() -> Client {
  rockbox.new()
  |> rockbox.host("localhost")
  |> rockbox.port(6062)
  |> rockbox.connect
}

/// Format a duration in milliseconds as `M:SS`.
pub fn fmt_ms(ms: Int) -> String {
  let total_secs = ms / 1000
  let mins = total_secs / 60
  let secs = total_secs % 60
  int.to_string(mins) <> ":" <> string.pad_start(int.to_string(secs), 2, "0")
}

/// Pad an integer to a left-aligned width-N string for tabular output.
pub fn pad_int(value: Int, width: Int) -> String {
  string.pad_start(int.to_string(value), width, " ")
}
