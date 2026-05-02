# 04 — Queue management
#
# Inspect the live playback queue and demonstrate insert / remove / clear.
#
#   mix run examples/04_queue_management.exs

Code.require_file("_helper.exs", __DIR__)

client = Examples.Helper.client()

{:ok, queue} = Rockbox.Queue.current(client)
IO.puts("Queue: #{queue.amount} tracks, currently at index #{queue.index}")

queue.tracks
|> Enum.with_index()
|> Enum.take(10)
|> Enum.each(fn {t, i} ->
  marker = if i == queue.index, do: "▶", else: " "
  IO.puts("#{marker} #{i + 1}. #{t.title} — #{t.artist}")
end)

# Pipe-friendly chained ops (uses bang variants):
#
#   client
#   |> tap(&Rockbox.Queue.clear!/1)
#   |> tap(&Rockbox.Queue.insert_tracks!(&1, ["/Music/a.mp3", "/Music/b.mp3"], :last))
#   |> Rockbox.Queue.shuffle!()
