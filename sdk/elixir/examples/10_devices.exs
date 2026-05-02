# 10 — List remote output devices (Chromecast / AirPlay / …)
#
#   mix run examples/10_devices.exs

Code.require_file("_helper.exs", __DIR__)

client = Examples.Helper.client()
{:ok, devices} = Rockbox.Devices.list(client)

IO.puts("Discovered #{length(devices)} device(s):")

for d <- devices do
  status = if d.is_connected, do: "● connected", else: "○ available"

  type =
    cond do
      d.is_cast_device -> "Cast"
      d.is_source_device -> "Source"
      true -> "Other"
    end

  IO.puts("  [#{type}] #{d.name} (#{d.ip}:#{d.port})  —  #{status}")
end
