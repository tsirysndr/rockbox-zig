# frozen_string_literal: true

require_relative "configuration"
require_relative "transport"
require_relative "events"
require_relative "plugin"
require_relative "types"

require_relative "api/playback"
require_relative "api/library"
require_relative "api/playlist"
require_relative "api/saved_playlists"
require_relative "api/smart_playlists"
require_relative "api/sound"
require_relative "api/settings"
require_relative "api/system"
require_relative "api/browse"
require_relative "api/devices"
require_relative "api/bluetooth"

module Rockbox
  # ---------------------------------------------------------------------------
  # Rockbox::Client — main entry point.
  #
  # Inspired by:
  #   Mopidy    — domain namespace API (client.playback.play, client.library.search)
  #   Jellyfin  — plugin install/uninstall lifecycle
  #   Kodi      — rich device + playlist management
  #
  # @example Quick start
  #   client = Rockbox::Client.new
  #   client.connect  # start WebSocket subscriptions
  #
  #   client.on(:track_changed) { |track| puts "Now playing: #{track.title}" }
  #
  #   results = client.library.search("dark side")
  #   client.playback.play_album(results.albums.first.id, shuffle: true)
  #
  # @example Builder DSL
  #   client = Rockbox::Client.build do |c|
  #     c.host = "192.168.1.42"
  #     c.port = 6062
  #   end
  # ---------------------------------------------------------------------------
  class Client
    attr_reader :playback, :library, :playlist, :saved_playlists, :smart_playlists,
                :sound, :settings, :system, :browse, :devices, :bluetooth,
                :configuration

    # Block-form constructor — yields a {Configuration} for tweaking.
    def self.build
      config = Configuration.new
      yield config if block_given?
      new(configuration: config)
    end

    # @param host         [String]  hostname or IP of rockboxd (default: "localhost")
    # @param port         [Integer] GraphQL port (default: 6062)
    # @param http_url     [String]  override the full HTTP URL
    # @param ws_url       [String]  override the full WebSocket URL
    # @param open_timeout [Integer] HTTP connect timeout (seconds)
    # @param read_timeout [Integer] HTTP read timeout (seconds)
    # @param configuration [Configuration] pre-built configuration (rare)
    def initialize(host: nil, port: nil, http_url: nil, ws_url: nil,
                   open_timeout: nil, read_timeout: nil, configuration: nil)
      @configuration = configuration || Configuration.new(
        host: host, port: port,
        http_url: http_url, ws_url: ws_url,
        open_timeout: open_timeout, read_timeout: read_timeout
      )

      @http = HttpTransport.new(
        @configuration.resolved_http_url,
        open_timeout: @configuration.open_timeout || HttpTransport::DEFAULT_OPEN_TIMEOUT,
        read_timeout: @configuration.read_timeout || HttpTransport::DEFAULT_READ_TIMEOUT
      )
      @ws = WsTransport.new(@configuration.resolved_ws_url)

      @events = EventEmitter.new
      @plugins = PluginRegistry.new
      @subscriptions = []

      @playback        = Api::Playback.new(@http)
      @library         = Api::Library.new(@http)
      @playlist        = Api::Playlist.new(@http)
      @saved_playlists = Api::SavedPlaylists.new(@http)
      @smart_playlists = Api::SmartPlaylists.new(@http)
      @sound           = Api::Sound.new(@http)
      @settings        = Api::Settings.new(@http)
      @system          = Api::System.new(@http)
      @browse          = Api::Browse.new(@http)
      @devices         = Api::Devices.new(@http)
      @bluetooth       = Api::Bluetooth.new(@http)
    end

    # ---------------------------------------------------------------------------
    # Events — block-friendly delegation to the EventEmitter
    # ---------------------------------------------------------------------------

    def on(event, &block);                     @events.on(event, &block);  self; end
    def once(event, &block);                   @events.once(event, &block); self; end
    def off(event, listener = nil, &block);    @events.off(event, listener, &block); self; end
    def emit(event, payload = nil);            @events.emit(event, payload);  self; end
    def remove_all_listeners(event = nil);     @events.remove_all_listeners(event); self; end

    # ---------------------------------------------------------------------------
    # Real-time subscriptions
    # ---------------------------------------------------------------------------

    # Open the WebSocket and subscribe to the three default streams. Idempotent.
    #
    # @return [self]
    def connect
      return self unless @subscriptions.empty?

      @subscriptions << @ws.subscribe(
        <<~GQL, nil,
          subscription CurrentlyPlaying {
            currentlyPlayingSong {
              id title artist album albumArt albumId artistId path length elapsed
            }
          }
        GQL
        next: ->(result) {
          payload = result[:data]&.dig(:currently_playing_song)
          @events.emit(:track_changed, Track.from_hash(payload)) if payload
        },
        error: ->(err) { @events.emit(:ws_error, wrap_error(err)) },
        complete: -> {}
      )

      @subscriptions << @ws.subscribe(
        "subscription PlaybackStatus { playbackStatus { status } }", nil,
        next: ->(result) {
          status = result[:data]&.dig(:playback_status, :status)
          @events.emit(:status_changed, status) unless status.nil?
        },
        error: ->(err) { @events.emit(:ws_error, wrap_error(err)) },
        complete: -> {}
      )

      @subscriptions << @ws.subscribe(
        <<~GQL, nil,
          subscription PlaylistChanged {
            playlistChanged {
              amount index maxPlaylistSize firstIndex lastInsertPos seed lastShuffledStart
              tracks { id title artist album path length albumArt }
            }
          }
        GQL
        next: ->(result) {
          payload = result[:data]&.dig(:playlist_changed)
          if payload
            tracks = Array(payload[:tracks]).map { |t| Track.from_hash(t) }
            playlist = Rockbox::Playlist.new(
              amount:              payload[:amount],
              index:               payload[:index],
              max_playlist_size:   payload[:max_playlist_size],
              first_index:         payload[:first_index],
              last_insert_pos:     payload[:last_insert_pos],
              seed:                payload[:seed],
              last_shuffled_start: payload[:last_shuffled_start],
              tracks:              tracks
            )
            @events.emit(:playlist_changed, playlist)
          end
        },
        error: ->(err) { @events.emit(:ws_error, wrap_error(err)) },
        complete: -> {}
      )

      @events.emit(:ws_open)
      self
    end

    # Tear down subscriptions and close the WebSocket.
    def disconnect
      @subscriptions.each { |unsub| unsub.call rescue nil }
      @subscriptions.clear
      @ws.dispose
      @events.emit(:ws_close)
      self
    end

    # ---------------------------------------------------------------------------
    # Plugin system
    # ---------------------------------------------------------------------------

    # @example
    #   client.use(MyScrobbler.new(api_key: "..."))
    def use(plugin)
      ctx = PluginContext.new(
        query: ->(gql, variables = nil) { @http.execute(gql, variables) },
        events: @events
      )
      @plugins.register(plugin, ctx)
      self
    end

    def unuse(name)
      @plugins.unregister(name)
      self
    end

    def installed_plugins
      @plugins.list
    end

    # ---------------------------------------------------------------------------
    # Raw escape hatch — for one-off GraphQL operations
    # ---------------------------------------------------------------------------

    # @param query     [String]
    # @param variables [Hash, nil]
    # @return [Hash] the snake-cased data object
    def query(query, variables = nil)
      @http.execute(query, variables)
    end

    private

    def wrap_error(err)
      err.is_a?(Exception) ? err : NetworkError.new(err.to_s)
    end
  end
end
