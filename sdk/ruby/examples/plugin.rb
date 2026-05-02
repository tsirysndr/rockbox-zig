# frozen_string_literal: true
#
# Plugin example — listens to track-change events and prints a one-line log.
#
#   ruby -Ilib examples/plugin.rb

require "rockbox"

class ConsoleScrobbler < Rockbox::Plugin
  def name;        "console-scrobbler" end
  def version;     "0.1.0"              end
  def description; "Logs every track change to stdout" end

  def install(ctx)
    ctx.events.on(:track_changed) do |track|
      puts "♪ #{Time.now.iso8601}  #{track.artist} — #{track.title}"
    end
  end
end

client = Rockbox::Client.new
client.use(ConsoleScrobbler.new)
client.connect

trap("INT") { client.disconnect; exit }
sleep
