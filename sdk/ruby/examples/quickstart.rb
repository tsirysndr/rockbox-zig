# frozen_string_literal: true
#
# Run with:  ruby -Ilib examples/quickstart.rb
#
# Make sure rockboxd is reachable on http://localhost:6062.

require "rockbox"

# Configure with the builder DSL.
client = Rockbox::Client.build do |c|
  c.host = ENV.fetch("ROCKBOX_HOST", "localhost")
  c.port = ENV.fetch("ROCKBOX_PORT", "6062").to_i
end

puts "Rockbox version: #{client.system.version}"

if (track = client.playback.current_track)
  puts "Now playing: #{track.title} — #{track.artist}"
  puts "  album: #{track.album}, length: #{track.length}ms, elapsed: #{track.elapsed}ms"
else
  puts "Nothing playing."
end

# Search the library.
results = client.library.search("dark side")
puts "Found #{results.albums.size} albums and #{results.tracks.size} tracks"

# Play the first matching album with shuffle on.
if (first = results.albums.first)
  client.playback.play_album(first.id, shuffle: true)
  puts "▶ #{first.title} by #{first.artist}"
end
