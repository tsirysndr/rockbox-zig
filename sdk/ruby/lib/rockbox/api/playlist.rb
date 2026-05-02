# frozen_string_literal: true

require_relative "../types"

module Rockbox
  module Api
    class Playlist
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

      # @return [Rockbox::Playlist]
      def current
        data = @http.execute(<<~GQL)
          #{TRACK_FIELDS}
          query CurrentPlaylist {
            playlistGetCurrent {
              amount index maxPlaylistSize firstIndex
              lastInsertPos seed lastShuffledStart
              tracks { ...TrackFields }
            }
          }
        GQL
        pl = data[:playlist_get_current] || {}
        Rockbox::Playlist.new(
          amount:              pl[:amount],
          index:               pl[:index],
          max_playlist_size:   pl[:max_playlist_size],
          first_index:         pl[:first_index],
          last_insert_pos:     pl[:last_insert_pos],
          seed:                pl[:seed],
          last_shuffled_start: pl[:last_shuffled_start],
          tracks:              Array(pl[:tracks]).map { |t| Track.from_hash(t) }
        )
      end

      def amount
        @http.execute("query PlaylistAmount { playlistAmount }")[:playlist_amount]
      end

      # ---------------------------------------------------------------------
      # Queue management
      # ---------------------------------------------------------------------

      # @param paths       [Array<String>] file paths or track IDs to insert.
      # @param position    [Integer] one of {Rockbox::InsertPosition} (default: NEXT).
      # @param playlist_id [String, nil] target playlist; nil for the active queue.
      def insert_tracks(paths, position: InsertPosition::NEXT, playlist_id: nil)
        @http.execute(
          "mutation InsertTracks($playlistId: String, $position: Int!, $tracks: [String!]!) { " \
          "insertTracks(playlistId: $playlistId, position: $position, tracks: $tracks) }",
          { playlist_id: playlist_id, position: position, tracks: paths }
        )
        nil
      end

      def insert_directory(directory, position: InsertPosition::LAST, playlist_id: nil)
        @http.execute(
          "mutation InsertDirectory($playlistId: String, $position: Int!, $directory: String!) { " \
          "insertDirectory(playlistId: $playlistId, position: $position, directory: $directory) }",
          { playlist_id: playlist_id, position: position, directory: directory }
        )
        nil
      end

      def insert_album(album_id, position: InsertPosition::LAST)
        @http.execute(
          "mutation InsertAlbum($albumId: String!, $position: Int!) { " \
          "insertAlbum(albumId: $albumId, position: $position) }",
          { album_id: album_id, position: position }
        )
        nil
      end

      def remove_track(index)
        @http.execute(
          "mutation RemoveTrack($index: Int!) { playlistRemoveTrack(index: $index) }",
          { index: index }
        )
        nil
      end

      def clear
        @http.execute("mutation ClearPlaylist { playlistRemoveAllTracks }")
        nil
      end

      def shuffle
        @http.execute("mutation ShufflePlaylist { shufflePlaylist }")
        nil
      end

      def create(name, tracks)
        @http.execute(
          "mutation CreatePlaylist($name: String!, $tracks: [String!]!) { " \
          "playlistCreate(name: $name, tracks: $tracks) }",
          { name: name, tracks: tracks }
        )
        nil
      end

      def start(start_index: nil, elapsed: nil, offset: nil)
        @http.execute(
          "mutation PlaylistStart($startIndex: Int, $elapsed: Int, $offset: Int) { " \
          "playlistStart(startIndex: $startIndex, elapsed: $elapsed, offset: $offset) }",
          { start_index: start_index, elapsed: elapsed, offset: offset }.compact
        )
        nil
      end

      def resume
        @http.execute("mutation PlaylistResume { playlistResume }")
        nil
      end
    end
  end
end
