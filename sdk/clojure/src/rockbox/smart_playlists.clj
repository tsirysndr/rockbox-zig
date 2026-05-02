(ns rockbox.smart-playlists
  "Smart (rule-based) playlists and listening stats."
  (:refer-clojure :exclude [list get update])
  (:require [clojure.data.json :as json]
            [rockbox.transport :as t]))

(def ^:private smart-fields
  "id name description image folderId isSystem rules createdAt updatedAt")

;; ---------------------------------------------------------------------------
;; Read
;; ---------------------------------------------------------------------------

(defn list [client]
  (:smart-playlists
   (t/execute client
              (str "query SmartPlaylists { smartPlaylists { " smart-fields " } }"))))

(defn get [client id]
  (:smart-playlist
   (t/execute client
              (str "query SmartPlaylist($id: String!) { smartPlaylist(id: $id) { " smart-fields " } }")
              {:id id})))

(defn track-ids
  "Resolve a smart playlist to the matching track ids right now."
  [client id]
  (:smart-playlist-track-ids
   (t/execute client
              "query SmartPlaylistTrackIds($id: String!) { smartPlaylistTrackIds(id: $id) }"
              {:id id})))

;; ---------------------------------------------------------------------------
;; Mutations — `:rules` may be a string or any data; data is JSON-encoded
;; ---------------------------------------------------------------------------

(defn- ->rules-string [rules]
  (cond
    (string? rules) rules
    (nil?    rules) nil
    :else           (json/write-str rules)))

(defn create
  "Create a smart playlist. `input` keys: `:name` (required), `:rules`
  (required — string or data), `:description`, `:image`, `:folder-id`."
  [client {:keys [name rules] :as input}]
  (when-not (and name rules)
    (throw (ex-info ":name and :rules are required"
                    {:type :rockbox/config :input input})))
  (:create-smart-playlist
   (t/execute client
              (str "mutation CreateSmartPlaylist(
                      $name: String!, $rules: String!, $description: String,
                      $image: String, $folderId: String
                    ) {
                      createSmartPlaylist(
                        name: $name, rules: $rules, description: $description,
                        image: $image, folderId: $folderId
                      ) { " smart-fields " }
                    }")
              (assoc input :rules (->rules-string rules)))))

(defn update
  "Update a smart playlist. `input` keys: `:name`, `:rules`, ..."
  [client id {:keys [name rules] :as input}]
  (when-not (and name rules)
    (throw (ex-info ":name and :rules are required"
                    {:type :rockbox/config :input input})))
  (t/execute client
             "mutation UpdateSmartPlaylist(
                $id: String!, $name: String!, $rules: String!,
                $description: String, $image: String, $folderId: String
              ) {
                updateSmartPlaylist(
                  id: $id, name: $name, rules: $rules, description: $description,
                  image: $image, folderId: $folderId
                )
              }"
             (assoc input :id id :rules (->rules-string rules)))
  client)

(defn delete [client id]
  (t/execute client "mutation DeleteSmartPlaylist($id: String!) { deleteSmartPlaylist(id: $id) }"
             {:id id})
  client)

(defn play [client id]
  (t/execute client
             "mutation PlaySmartPlaylist($id: String!) { playSmartPlaylist(id: $id) }"
             {:id id})
  client)

;; ---------------------------------------------------------------------------
;; Listening stats
;; ---------------------------------------------------------------------------

(defn track-stats
  "Listening stats for a track, or `nil`."
  [client track-id]
  (:track-stats
   (t/execute client
              "query TrackStats($trackId: String!) {
                 trackStats(trackId: $trackId) {
                   trackId playCount skipCount lastPlayed lastSkipped updatedAt
                 }
               }"
              {:track-id track-id})))

(defn record-played
  "Record that a track was played."
  [client track-id]
  (t/execute client "mutation RecordTrackPlayed($trackId: String!) { recordTrackPlayed(trackId: $trackId) }"
             {:track-id track-id})
  client)

(defn record-skipped
  "Record that a track was skipped."
  [client track-id]
  (t/execute client "mutation RecordTrackSkipped($trackId: String!) { recordTrackSkipped(trackId: $trackId) }"
             {:track-id track-id})
  client)
