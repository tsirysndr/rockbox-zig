# frozen_string_literal: true

require_relative "../types"

module Rockbox
  module Api
    class Playback
      TRACK_FIELDS = <<~GQL
        fragment TrackFields on Track {
          id title artist album genre disc trackString yearString
          composer comment albumArtist grouping
          discnum tracknum layer year bitrate frequency
          filesize length elapsed path
          albumId artistId genreId albumArt
        }
      GQL

      def initialize(http)
        @http = http
      end

      # ---------------------------------------------------------------------
      # Status & current track
      # ---------------------------------------------------------------------

      # @return [Integer] one of {Rockbox::PlaybackStatus} constants.
      def status
        @http.execute("query PlaybackStatus { status }")[:status]
      end

      # @return [Symbol] :stopped | :playing | :paused | :unknown
      def status_name
        PlaybackStatus.name(status)
      end

      # @return [Rockbox::Track, nil]
      def current_track
        data = @http.execute("#{TRACK_FIELDS}\nquery CurrentTrack { currentTrack { ...TrackFields } }")
        Track.from_hash(data[:current_track])
      end

      # @return [Rockbox::Track, nil]
      def next_track
        data = @http.execute("#{TRACK_FIELDS}\nquery NextTrack { nextTrack { ...TrackFields } }")
        Track.from_hash(data[:next_track])
      end

      # @return [Integer]
      def file_position
        @http.execute("query FilePosition { getFilePosition }")[:get_file_position]
      end

      # ---------------------------------------------------------------------
      # Transport controls
      # ---------------------------------------------------------------------

      def play(elapsed: 0, offset: 0)
        @http.execute(
          "mutation Play($elapsed: Long!, $offset: Long!) { play(elapsed: $elapsed, offset: $offset) }",
          { elapsed: elapsed, offset: offset }
        )
        nil
      end

      def pause;            @http.execute("mutation Pause { pause }");                            nil; end
      def resume;           @http.execute("mutation Resume { resume }");                          nil; end
      def next!;            @http.execute("mutation Next { next }");                              nil; end
      def previous!;        @http.execute("mutation Previous { previous }");                      nil; end
      def stop;             @http.execute("mutation Stop { hardStop }");                          nil; end
      def flush_and_reload; @http.execute("mutation FlushReload { flushAndReloadTracks }");       nil; end

      # @param position_ms [Integer] absolute target position, in milliseconds.
      def seek(position_ms)
        @http.execute(
          "mutation Seek($newTime: Int!) { fastForwardRewind(newTime: $newTime) }",
          { new_time: position_ms }
        )
        nil
      end

      # ---------------------------------------------------------------------
      # Single-call play helpers
      # ---------------------------------------------------------------------

      def play_track(path)
        @http.execute(
          "mutation PlayTrack($path: String!) { playTrack(path: $path) }",
          { path: path }
        )
        nil
      end

      def play_album(album_id, shuffle: nil, position: nil)
        @http.execute(
          "mutation PlayAlbum($albumId: String!, $shuffle: Boolean, $position: Int) { " \
          "playAlbum(albumId: $albumId, shuffle: $shuffle, position: $position) }",
          { album_id: album_id, shuffle: shuffle, position: position }.compact
        )
        nil
      end

      def play_artist(artist_id, shuffle: nil, position: nil)
        @http.execute(
          "mutation PlayArtist($artistId: String!, $shuffle: Boolean, $position: Int) { " \
          "playArtistTracks(artistId: $artistId, shuffle: $shuffle, position: $position) }",
          { artist_id: artist_id, shuffle: shuffle, position: position }.compact
        )
        nil
      end

      def play_playlist(playlist_id, shuffle: nil, position: nil)
        @http.execute(
          "mutation PlayPlaylist($playlistId: String!, $shuffle: Boolean, $position: Int) { " \
          "playPlaylist(playlistId: $playlistId, shuffle: $shuffle, position: $position) }",
          { playlist_id: playlist_id, shuffle: shuffle, position: position }.compact
        )
        nil
      end

      def play_directory(path, recurse: nil, shuffle: nil, position: nil)
        @http.execute(
          "mutation PlayDirectory($path: String!, $recurse: Boolean, $shuffle: Boolean, $position: Int) { " \
          "playDirectory(path: $path, recurse: $recurse, shuffle: $shuffle, position: $position) }",
          { path: path, recurse: recurse, shuffle: shuffle, position: position }.compact
        )
        nil
      end

      def play_liked_tracks(shuffle: nil, position: nil)
        @http.execute(
          "mutation PlayLikedTracks($shuffle: Boolean, $position: Int) { " \
          "playLikedTracks(shuffle: $shuffle, position: $position) }",
          { shuffle: shuffle, position: position }.compact
        )
        nil
      end

      def play_all_tracks(shuffle: nil, position: nil)
        @http.execute(
          "mutation PlayAllTracks($shuffle: Boolean, $position: Int) { " \
          "playAllTracks(shuffle: $shuffle, position: $position) }",
          { shuffle: shuffle, position: position }.compact
        )
        nil
      end
    end
  end
end
