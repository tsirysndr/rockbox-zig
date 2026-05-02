# 01 — Basic playback
#
# Inspect the current track, then either pause or resume based on the current
# state. Idempotent: run it twice and it toggles between Playing and Paused.
#
#   mix run examples/01_basic_playback.exs

Code.require_file("_helper.exs", __DIR__)

client = Examples.Helper.client()

{:ok, status} = Rockbox.Playback.status(client)
IO.puts("Status: #{status}")

case Rockbox.Playback.current_track(client) do
  {:ok, %Rockbox.Track{} = track} ->
    pct =
      if track.length > 0,
        do: round(track.elapsed / track.length * 100),
        else: 0

    IO.puts("Now: #{track.title} — #{track.artist}")

    IO.puts(
      "     #{Rockbox.Track.format_elapsed(track)} / #{Rockbox.Track.format_length(track)} (#{pct}%)"
    )

  {:ok, nil} ->
    IO.puts("Nothing is playing.")

  {:error, e} ->
    IO.puts(:stderr, "Error: #{Exception.message(e)}")
    System.halt(1)
end

case status do
  :playing ->
    :ok = Rockbox.Playback.pause(client)
    IO.puts("→ paused")

  :paused ->
    :ok = Rockbox.Playback.resume(client)
    IO.puts("→ resumed")

  _ ->
    :ok
end
