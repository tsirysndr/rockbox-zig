# 08 — EQ configuration
#
# Enable the equalizer with a 5-band bass-boost / presence-cut profile.
#
#   mix run examples/08_eq_config.exs

Code.require_file("_helper.exs", __DIR__)

client = Examples.Helper.client()

:ok =
  Rockbox.Settings.update(client,
    eq_enabled: true,
    eq_precut: -3,
    eq_band_settings: [
      %{cutoff: 60, q: 7, gain: 3},
      %{cutoff: 200, q: 7, gain: 0},
      %{cutoff: 800, q: 7, gain: 0},
      %{cutoff: 4000, q: 7, gain: -2},
      %{cutoff: 12_000, q: 7, gain: 1}
    ]
  )

{:ok, settings} = Rockbox.Settings.get(client)

IO.puts("EQ enabled: #{settings.eq_enabled}")
IO.puts("EQ precut:  #{settings.eq_precut}")

for band <- settings.eq_band_settings do
  IO.puts("  #{String.pad_leading(Integer.to_string(band.cutoff), 6)} Hz  q=#{band.q}  #{band.gain} dB")
end
