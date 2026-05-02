# 09 — Browse the filesystem
#
#   mix run examples/09_browse_filesystem.exs           # music_dir root
#   mix run examples/09_browse_filesystem.exs /Music/Pink\ Floyd

Code.require_file("_helper.exs", __DIR__)

client = Examples.Helper.client()
path = List.first(System.argv())

{:ok, entries} = Rockbox.Browse.entries(client, path)

for e <- entries do
  icon = if Rockbox.Entry.directory?(e), do: "[dir] ", else: "      "
  IO.puts("#{icon}#{e.name}")
end
