# frozen_string_literal: true

module Rockbox
  # Numeric playback states reported by the firmware.
  module PlaybackStatus
    STOPPED = 0
    PLAYING = 1
    PAUSED  = 3

    def self.name(value)
      { 0 => :stopped, 1 => :playing, 3 => :paused }[value] || :unknown
    end
  end

  module RepeatMode
    OFF       = 0
    ALL       = 1
    ONE       = 2
    SHUFFLE   = 3
    AB_REPEAT = 4
  end

  module ChannelConfig
    STEREO        = 0
    STEREO_NARROW = 1
    MONO          = 2
    LEFT_MIX      = 3
    RIGHT_MIX     = 4
    KARAOKE       = 5
  end

  module ReplaygainType
    TRACK   = 0
    ALBUM   = 1
    SHUFFLE = 2
  end

  # Where to insert tracks in the queue (Kodi/Mopidy convention).
  module InsertPosition
    NEXT          = 0
    AFTER_CURRENT = 1
    LAST          = 2
    FIRST         = 3
  end

  # ---------------------------------------------------------------------------
  # Value objects
  # ---------------------------------------------------------------------------
  #
  # Each type is a Struct that ignores unknown keys at construction so the
  # firmware can add fields without breaking older SDK builds.
  # ---------------------------------------------------------------------------

  module Type
    # Build a Struct that tolerates unknown / missing fields when coming
    # from a Hash. Struct subclasses raise on unknown keys with `keyword_init`,
    # so {.from_hash} filters the input.
    def self.with(*members)
      klass = Struct.new(*members, keyword_init: true)
      klass.define_singleton_method(:from_hash) do |hash|
        return nil if hash.nil?
        attrs = {}
        members.each { |m| attrs[m] = hash[m] }
        new(**attrs)
      end
      klass.define_singleton_method(:known_members) { members.dup }
      klass
    end
  end

  Track = Type.with(
    :id, :title, :artist, :album, :genre, :disc, :track_string, :year_string,
    :composer, :comment, :album_artist, :grouping,
    :discnum, :tracknum, :layer, :year, :bitrate, :frequency,
    :filesize, :length, :elapsed, :path,
    :album_id, :artist_id, :genre_id, :album_art
  )

  Album = Type.with(
    :id, :title, :artist, :year, :year_string, :album_art, :md5,
    :artist_id, :copyright_message, :tracks
  )

  Artist = Type.with(
    :id, :name, :bio, :image, :tracks, :albums
  )

  SearchResults = Type.with(
    :artists, :albums, :tracks, :liked_tracks, :liked_albums
  )

  Playlist = Type.with(
    :amount, :index, :max_playlist_size, :first_index,
    :last_insert_pos, :seed, :last_shuffled_start, :tracks
  )

  SavedPlaylist = Type.with(
    :id, :name, :description, :image, :folder_id,
    :track_count, :created_at, :updated_at
  )

  SavedPlaylistFolder = Type.with(:id, :name, :created_at, :updated_at)

  SmartPlaylist = Type.with(
    :id, :name, :description, :image, :folder_id, :is_system,
    :rules, :created_at, :updated_at
  )

  TrackStats = Type.with(
    :track_id, :play_count, :skip_count, :last_played, :last_skipped, :updated_at
  )

  BluetoothDevice = Type.with(
    :address, :name, :paired, :trusted, :connected, :rssi
  )

  VolumeInfo = Type.with(:volume, :min, :max)

  Device = Type.with(
    :id, :name, :host, :ip, :port, :service, :app, :is_connected,
    :base_url, :is_cast_device, :is_source_device, :is_current_device
  )

  Entry = Type.with(:name, :attr, :time_write, :customaction, :display_name)

  # File-attribute bit set on directory entries.
  ENTRY_DIR_BIT = 0x10

  def self.directory?(entry)
    (entry.attr.to_i & ENTRY_DIR_BIT) != 0
  end

  SystemStatus = Type.with(
    :resume_index, :resume_crc32, :resume_elapsed, :resume_offset,
    :runtime, :topruntime, :dircache_size,
    :last_screen, :viewer_icon_count, :last_volume_change
  )

  EqBandSetting       = Type.with(:cutoff, :q, :gain)
  ReplaygainSettings  = Type.with(:noclip, :type, :preamp)
  CompressorSettings  = Type.with(:threshold, :makeup_gain, :ratio, :knee, :release_time, :attack_time)

  UserSettings = Type.with(
    :music_dir, :volume, :balance, :bass, :treble, :channel_config, :stereo_width,
    :eq_enabled, :eq_precut, :eq_band_settings, :replaygain_settings, :compressor_settings,
    :crossfade_enabled, :crossfade_fade_in_delay, :crossfade_fade_in_duration,
    :crossfade_fade_out_delay, :crossfade_fade_out_duration, :crossfade_fade_out_mixmode,
    :crossfeed_enabled, :crossfeed_direct_gain, :crossfeed_cross_gain,
    :crossfeed_hf_attenuation, :crossfeed_hf_cutoff,
    :repeat_mode, :single_mode, :party_mode, :shuffle, :player_name
  )
end
