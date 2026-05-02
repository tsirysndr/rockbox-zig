# frozen_string_literal: true

require_relative "../types"

module Rockbox
  module Api
    class SavedPlaylists
      def initialize(http)
        @http = http
      end

      def list(folder_id: nil)
        data = @http.execute(
          "query SavedPlaylists($folderId: String) { " \
          "savedPlaylists(folderId: $folderId) { id name description image folderId trackCount createdAt updatedAt } }",
          { folder_id: folder_id }.compact
        )
        Array(data[:saved_playlists]).map { |p| SavedPlaylist.from_hash(p) }
      end

      def get(id)
        data = @http.execute(
          "query SavedPlaylist($id: String!) { " \
          "savedPlaylist(id: $id) { id name description image folderId trackCount createdAt updatedAt } }",
          { id: id }
        )
        SavedPlaylist.from_hash(data[:saved_playlist])
      end

      def track_ids(playlist_id)
        @http.execute(
          "query SavedPlaylistTrackIds($playlistId: String!) { savedPlaylistTrackIds(playlistId: $playlistId) }",
          { playlist_id: playlist_id }
        )[:saved_playlist_track_ids] || []
      end

      # @example Builder-friendly
      #   client.saved_playlists.create(name: "Late nights") do |p|
      #     p.description = "After-dark vibes"
      #     p.track_ids = ["abc", "def"]
      #   end
      def create(name:, description: nil, image: nil, folder_id: nil, track_ids: nil)
        builder = CreateBuilder.new(name, description, image, folder_id, track_ids)
        yield builder if block_given?

        data = @http.execute(<<~GQL, builder.to_variables)
          mutation CreateSavedPlaylist(
            $name: String!, $description: String, $image: String,
            $folderId: String, $trackIds: [String!]
          ) {
            createSavedPlaylist(
              name: $name, description: $description, image: $image,
              folderId: $folderId, trackIds: $trackIds
            ) {
              id name description image folderId trackCount createdAt updatedAt
            }
          }
        GQL
        SavedPlaylist.from_hash(data[:create_saved_playlist])
      end

      # @example
      #   client.saved_playlists.update("pl_123", name: "Renamed")
      def update(id, name:, description: nil, image: nil, folder_id: nil)
        @http.execute(<<~GQL, { id: id, name: name, description: description, image: image, folder_id: folder_id }.compact)
          mutation UpdateSavedPlaylist(
            $id: String!, $name: String!, $description: String,
            $image: String, $folderId: String
          ) {
            updateSavedPlaylist(
              id: $id, name: $name, description: $description,
              image: $image, folderId: $folderId
            )
          }
        GQL
        nil
      end

      def delete(id)
        @http.execute(
          "mutation DeleteSavedPlaylist($id: String!) { deleteSavedPlaylist(id: $id) }",
          { id: id }
        )
        nil
      end

      def add_tracks(playlist_id, track_ids)
        @http.execute(
          "mutation AddTracksToSavedPlaylist($playlistId: String!, $trackIds: [String!]!) { " \
          "addTracksToSavedPlaylist(playlistId: $playlistId, trackIds: $trackIds) }",
          { playlist_id: playlist_id, track_ids: track_ids }
        )
        nil
      end

      def remove_track(playlist_id, track_id)
        @http.execute(
          "mutation RemoveTrackFromSavedPlaylist($playlistId: String!, $trackId: String!) { " \
          "removeTrackFromSavedPlaylist(playlistId: $playlistId, trackId: $trackId) }",
          { playlist_id: playlist_id, track_id: track_id }
        )
        nil
      end

      def play(playlist_id)
        @http.execute(
          "mutation PlaySavedPlaylist($playlistId: String!) { playSavedPlaylist(playlistId: $playlistId) }",
          { playlist_id: playlist_id }
        )
        nil
      end

      # ---------------------------------------------------------------------
      # Folders
      # ---------------------------------------------------------------------

      def folders
        data = @http.execute("query PlaylistFolders { playlistFolders { id name createdAt updatedAt } }")
        Array(data[:playlist_folders]).map { |f| SavedPlaylistFolder.from_hash(f) }
      end

      def create_folder(name)
        data = @http.execute(
          "mutation CreatePlaylistFolder($name: String!) { " \
          "createPlaylistFolder(name: $name) { id name createdAt updatedAt } }",
          { name: name }
        )
        SavedPlaylistFolder.from_hash(data[:create_playlist_folder])
      end

      def delete_folder(id)
        @http.execute(
          "mutation DeletePlaylistFolder($id: String!) { deletePlaylistFolder(id: $id) }",
          { id: id }
        )
        nil
      end

      # Builder for #create — supports the optional yield-block DSL.
      class CreateBuilder
        attr_accessor :name, :description, :image, :folder_id, :track_ids

        def initialize(name, description, image, folder_id, track_ids)
          @name = name
          @description = description
          @image = image
          @folder_id = folder_id
          @track_ids = track_ids
        end

        def to_variables
          {
            name: name,
            description: description,
            image: image,
            folder_id: folder_id,
            track_ids: track_ids
          }.compact
        end
      end
    end
  end
end
