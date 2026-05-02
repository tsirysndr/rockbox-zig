# 02 — Real-time "Now Playing" stream
#
# Open the WebSocket and print track changes as they happen.
#
#   mix run --no-halt examples/02_now_playing.exs

Code.require_file("_helper.exs", __DIR__)

client = Examples.Helper.client()
{:ok, _pid} = Rockbox.connect(client)
:ok = Rockbox.subscribe([:track_changed, :status_changed])

IO.puts("Listening for events. Press Ctrl+C to exit.")

defmodule Loop do
  def run do
    receive do
      {:rockbox, :track_changed, %Rockbox.Track{} = t} ->
        IO.puts("▶ #{t.title} — #{t.artist}  (#{Rockbox.format_ms(t.length)})")
        run()

      {:rockbox, :status_changed, status} ->
        IO.puts("· status → #{status}")
        run()
    end
  end
end

Loop.run()
