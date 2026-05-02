# 03 — Library search
#
#   mix run examples/03_library_search.exs "dark side"

Code.require_file("_helper.exs", __DIR__)

term = List.first(System.argv()) || "love"
client = Examples.Helper.client()

{:ok, results} = Rockbox.Library.search(client, term)

IO.puts("Search: \"#{term}\"")
IO.puts("  Artists  (#{length(results.artists)}):")
for a <- Enum.take(results.artists, 5), do: IO.puts("    • #{a.name}")

IO.puts("  Albums   (#{length(results.albums)}):")

for a <- Enum.take(results.albums, 5),
    do: IO.puts("    • #{a.title} — #{a.artist} (#{a.year})")

IO.puts("  Tracks   (#{length(results.tracks)}):")

for t <- Enum.take(results.tracks, 5),
    do: IO.puts("    • #{t.title} — #{t.artist}")
