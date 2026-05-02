# 06 — Smart playlist with the Rules builder
#
#   mix run examples/06_smart_playlist.exs

Code.require_file("_helper.exs", __DIR__)

alias Rockbox.SmartPlaylist.Rules

client = Examples.Helper.client()

rules =
  Rules.all_of()
  |> Rules.where(:play_count, :gte, 1)
  |> Rules.sort(:play_count, :desc)
  |> Rules.limit(25)
  |> Rules.to_json()

{:ok, sp} =
  Rockbox.SmartPlaylists.create(client,
    name: "Most played (demo)",
    description: "Top 25 most-played tracks",
    rules: rules
  )

IO.puts("Created smart playlist: #{sp.id}")

{:ok, ids} = Rockbox.SmartPlaylists.track_ids(client, sp.id)
IO.puts("Currently resolves to #{length(ids)} tracks")

case List.first(ids) do
  nil ->
    :ok

  top_id ->
    {:ok, stats} = Rockbox.SmartPlaylists.track_stats(client, top_id)

    if stats do
      IO.puts(
        "Top track stats: played #{stats.play_count}× (skipped #{stats.skip_count}×)"
      )
    end
end

:ok = Rockbox.SmartPlaylists.delete(client, sp.id)
IO.puts("Cleaned up demo smart playlist.")
