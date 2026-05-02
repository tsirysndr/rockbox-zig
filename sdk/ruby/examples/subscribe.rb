# frozen_string_literal: true
#
# Subscribe to real-time playback events. Run with:
#   ruby -Ilib examples/subscribe.rb

require "rockbox"

client = Rockbox::Client.new

client.on(:track_changed) do |track|
  puts "▶ #{track.title} — #{track.artist}"
end

client.on(:status_changed) do |status|
  puts "status -> #{Rockbox::PlaybackStatus.name(status)}"
end

client.on(:playlist_changed) do |playlist|
  puts "playlist now has #{playlist.amount} tracks (index=#{playlist.index})"
end

client.on(:ws_error) { |err| warn "ws error: #{err.message}" }

client.connect
puts "Listening for events. Ctrl+C to exit."

trap("INT") do
  client.disconnect
  exit
end

sleep
