# frozen_string_literal: true

require_relative "../types"

module Rockbox
  module Api
    class SmartPlaylists
      def initialize(http)
        @http = http
      end

      def list
        data = @http.execute(<<~GQL)
          query SmartPlaylists {
            smartPlaylists { id name description image folderId isSystem rules createdAt updatedAt }
          }
        GQL
        Array(data[:smart_playlists]).map { |p| SmartPlaylist.from_hash(p) }
      end

      def get(id)
        data = @http.execute(
          "query SmartPlaylist($id: String!) { " \
          "smartPlaylist(id: $id) { id name description image folderId isSystem rules createdAt updatedAt } }",
          { id: id }
        )
        SmartPlaylist.from_hash(data[:smart_playlist])
      end

      def track_ids(id)
        @http.execute(
          "query SmartPlaylistTrackIds($id: String!) { smartPlaylistTrackIds(id: $id) }",
          { id: id }
        )[:smart_playlist_track_ids] || []
      end

      # @example
      #   client.smart_playlists.create(name: "Heavy hitters", rules: rules_json)
      def create(name:, rules:, description: nil, image: nil, folder_id: nil)
        vars = { name: name, rules: rules, description: description,
                 image: image, folder_id: folder_id }.compact
        data = @http.execute(<<~GQL, vars)
          mutation CreateSmartPlaylist(
            $name: String!, $rules: String!, $description: String,
            $image: String, $folderId: String
          ) {
            createSmartPlaylist(
              name: $name, rules: $rules, description: $description,
              image: $image, folderId: $folderId
            ) { id name description image folderId isSystem rules createdAt updatedAt }
          }
        GQL
        SmartPlaylist.from_hash(data[:create_smart_playlist])
      end

      def update(id, name:, rules:, description: nil, image: nil, folder_id: nil)
        vars = { id: id, name: name, rules: rules, description: description,
                 image: image, folder_id: folder_id }.compact
        @http.execute(<<~GQL, vars)
          mutation UpdateSmartPlaylist(
            $id: String!, $name: String!, $rules: String!,
            $description: String, $image: String, $folderId: String
          ) {
            updateSmartPlaylist(
              id: $id, name: $name, rules: $rules,
              description: $description, image: $image, folderId: $folderId
            )
          }
        GQL
        nil
      end

      def delete(id)
        @http.execute("mutation DeleteSmartPlaylist($id: String!) { deleteSmartPlaylist(id: $id) }", { id: id })
        nil
      end

      def play(id)
        @http.execute("mutation PlaySmartPlaylist($id: String!) { playSmartPlaylist(id: $id) }", { id: id })
        nil
      end

      # ---------------------------------------------------------------------
      # Listening stats
      # ---------------------------------------------------------------------

      def track_stats(track_id)
        data = @http.execute(<<~GQL, { track_id: track_id })
          query TrackStats($trackId: String!) {
            trackStats(trackId: $trackId) {
              trackId playCount skipCount lastPlayed lastSkipped updatedAt
            }
          }
        GQL
        TrackStats.from_hash(data[:track_stats])
      end

      def record_played(track_id)
        @http.execute(
          "mutation RecordTrackPlayed($trackId: String!) { recordTrackPlayed(trackId: $trackId) }",
          { track_id: track_id }
        )
        nil
      end

      def record_skipped(track_id)
        @http.execute(
          "mutation RecordTrackSkipped($trackId: String!) { recordTrackSkipped(trackId: $trackId) }",
          { track_id: track_id }
        )
        nil
      end
    end
  end
end
