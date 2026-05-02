# 11 — Bluetooth (Linux only)
#
#   mix run examples/11_bluetooth.exs                    # list paired devices
#   mix run examples/11_bluetooth.exs scan               # scan 10s for devices
#   mix run examples/11_bluetooth.exs connect AA:BB:..   # connect by address
#   mix run examples/11_bluetooth.exs disconnect AA:BB:..

Code.require_file("_helper.exs", __DIR__)

client = Examples.Helper.client()

format = fn d ->
  flags =
    [
      if(d.connected, do: "connected"),
      if(d.paired, do: "paired"),
      if(d.trusted, do: "trusted")
    ]
    |> Enum.reject(&is_nil/1)
    |> Enum.join(", ")

  rssi = if d.rssi, do: " #{d.rssi} dBm", else: ""
  "  #{d.address}  #{String.pad_trailing(d.name || "", 28)}#{rssi}  [#{flags}]"
end

case System.argv() do
  ["scan"] ->
    IO.puts("Scanning for 10s...")
    {:ok, found} = Rockbox.Bluetooth.scan(client, 10)
    IO.puts("Found #{length(found)} devices:")
    for d <- found, do: IO.puts(format.(d))

  ["connect", addr] ->
    :ok = Rockbox.Bluetooth.connect(client, addr)
    IO.puts("Connected to #{addr}")

  ["disconnect", addr] ->
    :ok = Rockbox.Bluetooth.disconnect(client, addr)
    IO.puts("Disconnected from #{addr}")

  _ ->
    {:ok, devices} = Rockbox.Bluetooth.devices(client)
    IO.puts("Paired devices (#{length(devices)}):")
    for d <- devices, do: IO.puts(format.(d))
end
