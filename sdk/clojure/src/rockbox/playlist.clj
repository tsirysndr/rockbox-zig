(ns rockbox.playlist
  "Live playback queue management — what's currently playing and what's
  queued up next. For persistent named playlists see `rockbox.saved-playlists`."
  (:refer-clojure :exclude [shuffle])
  (:require [rockbox.transport :as t]
            [rockbox.types     :as types]))

(def ^:private track-fields
  "fragment TrackFields on Track {
     id title artist album genre disc trackString yearString
     composer comment albumArtist grouping
     discnum tracknum layer year bitrate frequency
     filesize length elapsed path
     albumId artistId genreId albumArt
   }")

(defn current
  "The active queue: `:tracks :amount :index :max-playlist-size ...`."
  [client]
  (:playlist-get-current
   (t/execute client
              (str track-fields
                   " query CurrentPlaylist {
                       playlistGetCurrent {
                         amount index maxPlaylistSize firstIndex
                         lastInsertPos seed lastShuffledStart
                         tracks { ...TrackFields }
                       }
                     }"))))

(defn amount
  "Number of tracks in the active queue."
  [client]
  (:playlist-amount (t/execute client "query PlaylistAmount { playlistAmount }")))

;; ---------------------------------------------------------------------------
;; Queue management
;; ---------------------------------------------------------------------------

(defn insert-tracks
  "Insert track paths (or IDs) into the queue.

  `position` is a `rockbox.types/insert-position` keyword (`:next :after-current
  :last :first`) or the underlying integer; defaults to `:next`.

  Optional `:playlist-id` targets a specific playlist instead of the active queue."
  ([client paths] (insert-tracks client paths :next nil))
  ([client paths position] (insert-tracks client paths position nil))
  ([client paths position playlist-id]
   (t/execute client
              "mutation InsertTracks($playlistId: String, $position: Int!, $tracks: [String!]!) {
                 insertTracks(playlistId: $playlistId, position: $position, tracks: $tracks)
               }"
              {:playlist-id playlist-id
               :position    (types/->insert-position position)
               :tracks      (vec paths)})
   client))

(defn insert-directory
  "Insert a directory's contents (recursively) into the queue."
  ([client directory] (insert-directory client directory :last nil))
  ([client directory position] (insert-directory client directory position nil))
  ([client directory position playlist-id]
   (t/execute client
              "mutation InsertDirectory($playlistId: String, $position: Int!, $directory: String!) {
                 insertDirectory(playlistId: $playlistId, position: $position, directory: $directory)
               }"
              {:playlist-id playlist-id
               :position    (types/->insert-position position)
               :directory   directory})
   client))

(defn insert-album
  "Append all tracks from an album to the queue."
  ([client album-id] (insert-album client album-id :last))
  ([client album-id position]
   (t/execute client
              "mutation InsertAlbum($albumId: String!, $position: Int!) {
                 insertAlbum(albumId: $albumId, position: $position)
               }"
              {:album-id album-id :position (types/->insert-position position)})
   client))

(defn remove-track
  "Remove the track at queue index `i` (0-based)."
  [client i]
  (t/execute client "mutation RemoveTrack($index: Int!) { playlistRemoveTrack(index: $index) }"
             {:index i})
  client)

(defn clear
  "Remove every track from the queue."
  [client]
  (t/execute client "mutation ClearPlaylist { playlistRemoveAllTracks }")
  client)

(defn shuffle
  "Shuffle the remaining tracks in the queue."
  [client]
  (t/execute client "mutation ShufflePlaylist { shufflePlaylist }")
  client)

(defn create
  "Create and start a new temporary queue from a list of paths.
  Replaces the current queue."
  [client name paths]
  (t/execute client
             "mutation CreatePlaylist($name: String!, $tracks: [String!]!) {
                playlistCreate(name: $name, tracks: $tracks)
              }"
             {:name name :tracks (vec paths)})
  client)

(defn start
  "Start playback of the current queue. Options:
   `:start-index` (int), `:elapsed` (int), `:offset` (int)."
  ([client] (start client {}))
  ([client opts]
   (t/execute client
              "mutation PlaylistStart($startIndex: Int, $elapsed: Int, $offset: Int) {
                 playlistStart(startIndex: $startIndex, elapsed: $elapsed, offset: $offset)
               }"
              opts)
   client))

(defn resume
  "Resume from the saved position."
  [client]
  (t/execute client "mutation PlaylistResume { playlistResume }")
  client)
