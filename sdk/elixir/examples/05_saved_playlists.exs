# 05 — Saved playlists
#
#   mix run examples/05_saved_playlists.exs

Code.require_file("_helper.exs", __DIR__)

client = Examples.Helper.client()

{:ok, lists} = Rockbox.SavedPlaylists.list(client)
IO.puts("You have #{length(lists)} saved playlist(s):")

for pl <- lists do
  IO.puts("  • #{pl.name}  —  #{pl.track_count} tracks  (id: #{pl.id})")
end

# Create + add tracks + delete (uncomment to demo):
#
#   {:ok, pl} = Rockbox.SavedPlaylists.create(client, name: "Demo", description: "test")
#   :ok = Rockbox.SavedPlaylists.add_tracks(client, pl.id, ["track-id-1", "track-id-2"])
#   :ok = Rockbox.SavedPlaylists.delete(client, pl.id)
