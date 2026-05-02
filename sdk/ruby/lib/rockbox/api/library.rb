# frozen_string_literal: true

require_relative "../types"

module Rockbox
  module Api
    class Library
      TRACK_FIELDS = <<~GQL
        fragment TrackFields on Track {
          id title artist album genre disc trackString yearString
          composer comment albumArtist grouping
          discnum tracknum layer year bitrate frequency
          filesize length elapsed path
          albumId artistId genreId albumArt
        }
      GQL

      ALBUM_FIELDS = <<~GQL
        fragment AlbumFields on Album {
          id title artist year yearString albumArt md5 artistId copyrightMessage
        }
      GQL

      ARTIST_FIELDS = <<~GQL
        fragment ArtistFields on Artist { id name bio image }
      GQL

      def initialize(http)
        @http = http
      end

      # ---------------------------------------------------------------------
      # Albums
      # ---------------------------------------------------------------------

      def albums
        data = @http.execute(<<~GQL)
          #{ALBUM_FIELDS}
          query Albums { albums { ...AlbumFields tracks { id title path length albumArt } } }
        GQL
        Array(data[:albums]).map { |a| Album.from_hash(a) }
      end

      def album(id)
        data = @http.execute(<<~GQL, { id: id })
          #{TRACK_FIELDS}
          #{ALBUM_FIELDS}
          query Album($id: String!) { album(id: $id) { ...AlbumFields tracks { ...TrackFields } } }
        GQL
        Album.from_hash(data[:album])
      end

      def liked_albums
        data = @http.execute("#{ALBUM_FIELDS}\nquery LikedAlbums { likedAlbums { ...AlbumFields } }")
        Array(data[:liked_albums]).map { |a| Album.from_hash(a) }
      end

      def like_album(id)
        @http.execute("mutation LikeAlbum($id: String!) { likeAlbum(id: $id) }", { id: id })
        nil
      end

      def unlike_album(id)
        @http.execute("mutation UnlikeAlbum($id: String!) { unlikeAlbum(id: $id) }", { id: id })
        nil
      end

      # ---------------------------------------------------------------------
      # Artists
      # ---------------------------------------------------------------------

      def artists
        data = @http.execute(<<~GQL)
          #{ARTIST_FIELDS}
          query Artists { artists { ...ArtistFields albums { id title albumArt year } } }
        GQL
        Array(data[:artists]).map { |a| Artist.from_hash(a) }
      end

      def artist(id)
        data = @http.execute(<<~GQL, { id: id })
          #{ARTIST_FIELDS}
          #{TRACK_FIELDS}
          query Artist($id: String!) {
            artist(id: $id) {
              ...ArtistFields
              albums { id title albumArt year yearString md5 artistId tracks { id title path length } }
              tracks { ...TrackFields }
            }
          }
        GQL
        Artist.from_hash(data[:artist])
      end

      # ---------------------------------------------------------------------
      # Tracks
      # ---------------------------------------------------------------------

      def tracks
        data = @http.execute("#{TRACK_FIELDS}\nquery Tracks { tracks { ...TrackFields } }")
        Array(data[:tracks]).map { |t| Track.from_hash(t) }
      end

      def track(id)
        data = @http.execute(
          "#{TRACK_FIELDS}\nquery Track($id: String!) { track(id: $id) { ...TrackFields } }",
          { id: id }
        )
        Track.from_hash(data[:track])
      end

      def liked_tracks
        data = @http.execute("#{TRACK_FIELDS}\nquery LikedTracks { likedTracks { ...TrackFields } }")
        Array(data[:liked_tracks]).map { |t| Track.from_hash(t) }
      end

      def like_track(id)
        @http.execute("mutation LikeTrack($id: String!) { likeTrack(id: $id) }", { id: id })
        nil
      end

      def unlike_track(id)
        @http.execute("mutation UnlikeTrack($id: String!) { unlikeTrack(id: $id) }", { id: id })
        nil
      end

      # ---------------------------------------------------------------------
      # Search
      # ---------------------------------------------------------------------

      # @return [Rockbox::SearchResults]
      def search(term)
        data = @http.execute(<<~GQL, { term: term })
          #{TRACK_FIELDS}
          #{ALBUM_FIELDS}
          #{ARTIST_FIELDS}
          query Search($term: String!) {
            search(term: $term) {
              artists { ...ArtistFields }
              albums { ...AlbumFields }
              tracks { ...TrackFields }
              likedTracks { ...TrackFields }
              likedAlbums { ...AlbumFields }
            }
          }
        GQL
        results = data[:search] || {}
        SearchResults.new(
          artists:      Array(results[:artists]).map      { |a| Artist.from_hash(a) },
          albums:       Array(results[:albums]).map       { |a| Album.from_hash(a) },
          tracks:       Array(results[:tracks]).map       { |t| Track.from_hash(t) },
          liked_tracks: Array(results[:liked_tracks]).map { |t| Track.from_hash(t) },
          liked_albums: Array(results[:liked_albums]).map { |a| Album.from_hash(a) }
        )
      end

      # ---------------------------------------------------------------------
      # Library management
      # ---------------------------------------------------------------------

      def scan
        @http.execute("mutation ScanLibrary { scanLibrary }")
        nil
      end
    end
  end
end
