(ns rockbox.library
  "Library queries (albums, artists, tracks, search) and likes."
  (:require [rockbox.transport :as t]))

(def ^:private track-fields
  "fragment TrackFields on Track {
     id title artist album genre disc trackString yearString
     composer comment albumArtist grouping
     discnum tracknum layer year bitrate frequency
     filesize length elapsed path
     albumId artistId genreId albumArt
   }")

(def ^:private album-fields
  "fragment AlbumFields on Album {
     id title artist year yearString albumArt md5 artistId copyrightMessage
   }")

(def ^:private artist-fields
  "fragment ArtistFields on Artist { id name bio image }")

;; ---------------------------------------------------------------------------
;; Albums
;; ---------------------------------------------------------------------------

(defn albums
  "All albums (with shallow track stubs)."
  [client]
  (:albums (t/execute client (str album-fields
                                  " query Albums { albums { ...AlbumFields tracks { id title path length albumArt } } }"))))

(defn album
  "Single album with full track list, or `nil` if not found."
  [client id]
  (:album (t/execute client
                     (str track-fields album-fields
                          " query Album($id: String!) { album(id: $id) { ...AlbumFields tracks { ...TrackFields } } }")
                     {:id id})))

(defn liked-albums [client]
  (:liked-albums
   (t/execute client (str album-fields
                          " query LikedAlbums { likedAlbums { ...AlbumFields } }"))))

(defn like-album
  "Like an album. Returns the client."
  [client id]
  (t/execute client "mutation LikeAlbum($id: String!) { likeAlbum(id: $id) }" {:id id})
  client)

(defn unlike-album
  "Unlike an album. Returns the client."
  [client id]
  (t/execute client "mutation UnlikeAlbum($id: String!) { unlikeAlbum(id: $id) }" {:id id})
  client)

;; ---------------------------------------------------------------------------
;; Artists
;; ---------------------------------------------------------------------------

(defn artists
  "All artists (with shallow album stubs)."
  [client]
  (:artists
   (t/execute client (str artist-fields
                          " query Artists { artists { ...ArtistFields albums { id title albumArt year } } }"))))

(defn artist
  "Single artist with albums and tracks, or `nil`."
  [client id]
  (:artist
   (t/execute client
              (str artist-fields track-fields
                   " query Artist($id: String!) {
                       artist(id: $id) {
                         ...ArtistFields
                         albums { id title albumArt year yearString md5 artistId tracks { id title path length } }
                         tracks { ...TrackFields }
                       }
                     }")
              {:id id})))

;; ---------------------------------------------------------------------------
;; Tracks
;; ---------------------------------------------------------------------------

(defn tracks
  "All tracks in the library."
  [client]
  (:tracks (t/execute client (str track-fields
                                  " query Tracks { tracks { ...TrackFields } }"))))

(defn track
  "Single track by id, or `nil`."
  [client id]
  (:track (t/execute client (str track-fields
                                 " query Track($id: String!) { track(id: $id) { ...TrackFields } }")
                     {:id id})))

(defn liked-tracks [client]
  (:liked-tracks
   (t/execute client (str track-fields
                          " query LikedTracks { likedTracks { ...TrackFields } }"))))

(defn like-track
  "Like a track. Returns the client."
  [client id]
  (t/execute client "mutation LikeTrack($id: String!) { likeTrack(id: $id) }" {:id id})
  client)

(defn unlike-track
  "Unlike a track. Returns the client."
  [client id]
  (t/execute client "mutation UnlikeTrack($id: String!) { unlikeTrack(id: $id) }" {:id id})
  client)

;; ---------------------------------------------------------------------------
;; Search
;; ---------------------------------------------------------------------------

(defn search
  "Full-text search across artists, albums, tracks. Returns a map with
  `:artists :albums :tracks :liked-tracks :liked-albums`."
  [client term]
  (:search
   (t/execute client
              (str track-fields album-fields artist-fields
                   " query Search($term: String!) {
                       search(term: $term) {
                         artists { ...ArtistFields }
                         albums { ...AlbumFields }
                         tracks { ...TrackFields }
                         likedTracks { ...TrackFields }
                         likedAlbums { ...AlbumFields }
                       }
                     }")
              {:term term})))

;; ---------------------------------------------------------------------------
;; Library scan
;; ---------------------------------------------------------------------------

(defn scan
  "Trigger a full rescan of `music_dir`. Returns the client."
  [client]
  (t/execute client "mutation ScanLibrary { scanLibrary }")
  client)
