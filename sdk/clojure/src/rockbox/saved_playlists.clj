(ns rockbox.saved-playlists
  "Persistent named playlists stored in the database, with folder support."
  (:refer-clojure :exclude [list get update remove])
  (:require [rockbox.transport :as t]))

(def ^:private playlist-fields
  "id name description image folderId trackCount createdAt updatedAt")

;; ---------------------------------------------------------------------------
;; Read
;; ---------------------------------------------------------------------------

(defn list
  "List saved playlists, optionally filtered by folder."
  ([client] (list client nil))
  ([client folder-id]
   (:saved-playlists
    (t/execute client
               (str "query SavedPlaylists($folderId: String) {
                       savedPlaylists(folderId: $folderId) { " playlist-fields " }
                     }")
               {:folder-id folder-id}))))

(defn get
  "Single playlist by id, or `nil`."
  [client id]
  (:saved-playlist
   (t/execute client
              (str "query SavedPlaylist($id: String!) {
                      savedPlaylist(id: $id) { " playlist-fields " }
                    }")
              {:id id})))

(defn track-ids
  "Ordered track ids for a saved playlist."
  [client playlist-id]
  (:saved-playlist-track-ids
   (t/execute client
              "query SavedPlaylistTrackIds($playlistId: String!) {
                 savedPlaylistTrackIds(playlistId: $playlistId)
               }"
              {:playlist-id playlist-id})))

;; ---------------------------------------------------------------------------
;; Mutations
;; ---------------------------------------------------------------------------

(defn create
  "Create a saved playlist. `input` keys: `:name` (required), `:description`,
  `:image`, `:folder-id`, `:track-ids`. Returns the new playlist."
  [client {:keys [name] :as input}]
  (when-not name
    (throw (ex-info ":name is required" {:type :rockbox/config :input input})))
  (:create-saved-playlist
   (t/execute client
              (str "mutation CreateSavedPlaylist(
                      $name: String!, $description: String, $image: String,
                      $folderId: String, $trackIds: [String!]
                    ) {
                      createSavedPlaylist(
                        name: $name, description: $description, image: $image,
                        folderId: $folderId, trackIds: $trackIds
                      ) { " playlist-fields " }
                    }")
              input)))

(defn update
  "Update a saved playlist's metadata. `input`: `:name` (required),
  `:description`, `:image`, `:folder-id`. Returns the client."
  [client id {:keys [name] :as input}]
  (when-not name
    (throw (ex-info ":name is required" {:type :rockbox/config :input input})))
  (t/execute client
             "mutation UpdateSavedPlaylist(
                $id: String!, $name: String!, $description: String, $image: String, $folderId: String
              ) {
                updateSavedPlaylist(
                  id: $id, name: $name, description: $description, image: $image, folderId: $folderId
                )
              }"
             (assoc input :id id))
  client)

(defn delete
  "Delete a saved playlist."
  [client id]
  (t/execute client "mutation DeleteSavedPlaylist($id: String!) { deleteSavedPlaylist(id: $id) }"
             {:id id})
  client)

(defn add-tracks
  "Append tracks (by id) to a saved playlist."
  [client playlist-id track-ids]
  (t/execute client
             "mutation AddTracksToSavedPlaylist($playlistId: String!, $trackIds: [String!]!) {
                addTracksToSavedPlaylist(playlistId: $playlistId, trackIds: $trackIds)
              }"
             {:playlist-id playlist-id :track-ids (vec track-ids)})
  client)

(defn remove-track
  "Remove a single track from a saved playlist."
  [client playlist-id track-id]
  (t/execute client
             "mutation RemoveTrackFromSavedPlaylist($playlistId: String!, $trackId: String!) {
                removeTrackFromSavedPlaylist(playlistId: $playlistId, trackId: $trackId)
              }"
             {:playlist-id playlist-id :track-id track-id})
  client)

(defn play
  "Load a saved playlist into the queue and start playing."
  [client playlist-id]
  (t/execute client
             "mutation PlaySavedPlaylist($playlistId: String!) { playSavedPlaylist(playlistId: $playlistId) }"
             {:playlist-id playlist-id})
  client)

;; ---------------------------------------------------------------------------
;; Folders
;; ---------------------------------------------------------------------------

(defn folders
  "List playlist folders."
  [client]
  (:playlist-folders
   (t/execute client
              "query PlaylistFolders { playlistFolders { id name createdAt updatedAt } }")))

(defn create-folder
  "Create a new folder. Returns the folder map."
  [client name]
  (:create-playlist-folder
   (t/execute client
              "mutation CreatePlaylistFolder($name: String!) {
                 createPlaylistFolder(name: $name) { id name createdAt updatedAt }
               }"
              {:name name})))

(defn delete-folder
  "Delete a folder by id."
  [client id]
  (t/execute client "mutation DeletePlaylistFolder($id: String!) { deletePlaylistFolder(id: $id) }"
             {:id id})
  client)
