(ns rockbox.playback
  "Transport controls and one-call play helpers.

      (-> client
          (pb/play-album \"album-id\" {:shuffle true})
          (pb/seek 90000))

  Action functions return the client so they compose with `->`. Read
  functions return data."
  (:refer-clojure :exclude [next])
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

;; ---------------------------------------------------------------------------
;; State
;; ---------------------------------------------------------------------------

(defn raw-status
  "Raw integer playback status (firmware enum)."
  [client]
  (:status (t/execute client "query PlaybackStatus { status }")))

(defn status
  "Playback status as a keyword: `:stopped`, `:playing`, `:paused`."
  [client]
  (types/playback-status->keyword (raw-status client)))

(defn current-track
  "Currently playing track as a kebab-case map, or `nil` when stopped."
  [client]
  (:current-track
   (t/execute client (str track-fields
                          " query CurrentTrack { currentTrack { ...TrackFields } }"))))

(defn next-track
  "Next track in the queue, or `nil`."
  [client]
  (:next-track
   (t/execute client (str track-fields
                          " query NextTrack { nextTrack { ...TrackFields } }"))))

(defn file-position
  "Byte offset into the current file."
  [client]
  (:get-file-position (t/execute client "query FilePosition { getFilePosition }")))

;; ---------------------------------------------------------------------------
;; Transport controls — return the client (pipe-friendly)
;; ---------------------------------------------------------------------------

(defn play
  "Resume playback from queued position. Optional `:elapsed` and `:offset`."
  ([client] (play client {}))
  ([client {:keys [elapsed offset] :or {elapsed 0 offset 0}}]
   (t/execute client
              "mutation Play($elapsed: Long!, $offset: Long!) { play(elapsed: $elapsed, offset: $offset) }"
              {:elapsed elapsed :offset offset})
   client))

(defn pause [client]
  (t/execute client "mutation Pause { pause }") client)

(defn resume [client]
  (t/execute client "mutation Resume { resume }") client)

(defn next [client]
  (t/execute client "mutation Next { next }") client)

(defn previous [client]
  (t/execute client "mutation Previous { previous }") client)

(defn stop [client]
  (t/execute client "mutation Stop { hardStop }") client)

(defn flush-and-reload
  "Force-reload the current queue from disk."
  [client]
  (t/execute client "mutation FlushReload { flushAndReloadTracks }") client)

(defn seek
  "Seek to an absolute position in milliseconds."
  [client position-ms]
  (t/execute client
             "mutation Seek($newTime: Int!) { fastForwardRewind(newTime: $newTime) }"
             {:new-time position-ms})
  client)

;; ---------------------------------------------------------------------------
;; Play helpers
;; ---------------------------------------------------------------------------

(defn play-track
  "Play a single file by absolute path."
  [client path]
  (t/execute client
             "mutation PlayTrack($path: String!) { playTrack(path: $path) }"
             {:path path})
  client)

(defn play-album
  "Play all tracks from an album. Options: `:shuffle` (bool), `:position` (int)."
  ([client album-id] (play-album client album-id {}))
  ([client album-id opts]
   (t/execute client
              "mutation PlayAlbum($albumId: String!, $shuffle: Boolean, $position: Int) {
                 playAlbum(albumId: $albumId, shuffle: $shuffle, position: $position)
               }"
              (merge {:album-id album-id} opts))
   client))

(defn play-artist
  "Play all tracks by an artist. Options: `:shuffle`, `:position`."
  ([client artist-id] (play-artist client artist-id {}))
  ([client artist-id opts]
   (t/execute client
              "mutation PlayArtist($artistId: String!, $shuffle: Boolean, $position: Int) {
                 playArtistTracks(artistId: $artistId, shuffle: $shuffle, position: $position)
               }"
              (merge {:artist-id artist-id} opts))
   client))

(defn play-playlist
  "Play a saved playlist by id. Options: `:shuffle`, `:position`."
  ([client playlist-id] (play-playlist client playlist-id {}))
  ([client playlist-id opts]
   (t/execute client
              "mutation PlayPlaylist($playlistId: String!, $shuffle: Boolean, $position: Int) {
                 playPlaylist(playlistId: $playlistId, shuffle: $shuffle, position: $position)
               }"
              (merge {:playlist-id playlist-id} opts))
   client))

(defn play-directory
  "Play every file under a directory. Options: `:recurse`, `:shuffle`, `:position`."
  ([client path] (play-directory client path {}))
  ([client path opts]
   (t/execute client
              "mutation PlayDirectory($path: String!, $recurse: Boolean, $shuffle: Boolean, $position: Int) {
                 playDirectory(path: $path, recurse: $recurse, shuffle: $shuffle, position: $position)
               }"
              (merge {:path path} opts))
   client))

(defn play-liked-tracks
  "Play every liked track. Options: `:shuffle`, `:position`."
  ([client] (play-liked-tracks client {}))
  ([client opts]
   (t/execute client
              "mutation PlayLikedTracks($shuffle: Boolean, $position: Int) {
                 playLikedTracks(shuffle: $shuffle, position: $position)
               }"
              opts)
   client))

(defn play-all-tracks
  "Play the entire library. Almost always used with `:shuffle true`."
  ([client] (play-all-tracks client {}))
  ([client opts]
   (t/execute client
              "mutation PlayAllTracks($shuffle: Boolean, $position: Int) {
                 playAllTracks(shuffle: $shuffle, position: $position)
               }"
              opts)
   client))
