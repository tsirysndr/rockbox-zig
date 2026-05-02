# 07 — Volume control
#
#   mix run examples/07_volume_control.exs

Code.require_file("_helper.exs", __DIR__)

client = Examples.Helper.client()

{:ok, vol} = Rockbox.Sound.volume(client)
IO.puts("Current: #{vol.volume} (range #{vol.min}..#{vol.max})")

{:ok, after_up} = Rockbox.Sound.up(client)
IO.puts("After +1: #{after_up}")

{:ok, _} = Rockbox.Sound.down(client)
IO.puts("Reverted.")
