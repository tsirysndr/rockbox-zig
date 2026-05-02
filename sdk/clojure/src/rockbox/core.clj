(ns rockbox.core
  "Idiomatic Clojure SDK for [Rockbox](https://www.rockbox.org).

  ## Quick start

      (require '[rockbox.core :as rb]
               '[rockbox.playback :as pb]
               '[rockbox.library  :as lib])

      (def client (rb/client))

      ;; Optional: open the WebSocket for real-time events
      (rb/connect client)
      (rb/on client :track-changed
        (fn [t] (println \"▶\" (:title t) \"—\" (:artist t))))

      ;; Look at what's playing
      (when-let [t (pb/current-track client)]
        (println (:title t)))

      ;; Pipe-friendly: actions return the client so they compose
      (-> client
          (pb/pause)
          (pb/seek 90000)
          (pb/resume))

  ## Module map

  | Domain                | Namespace                  |
  |-----------------------|----------------------------|
  | Transport controls    | `rockbox.playback`         |
  | Library / search      | `rockbox.library`          |
  | Live queue            | `rockbox.playlist`         |
  | Saved playlists       | `rockbox.saved-playlists`  |
  | Smart playlists       | `rockbox.smart-playlists`  |
  | Volume                | `rockbox.sound`            |
  | Settings              | `rockbox.settings`         |
  | System info           | `rockbox.system`           |
  | Filesystem browser    | `rockbox.browse`           |
  | Output devices        | `rockbox.devices`          |
  | Bluetooth (Linux)     | `rockbox.bluetooth`        |
  | Real-time events      | `rockbox.events`           |
  | Plugin system         | `rockbox.plugin`           |
  | Enums and helpers     | `rockbox.types`            |"
  (:require [rockbox.transport :as transport]
            [rockbox.events    :as events]
            [rockbox.plugin    :as plugin]
            [rockbox.ws        :as ws]))

;; ---------------------------------------------------------------------------
;; Client construction
;; ---------------------------------------------------------------------------

(def ^:const default-host "localhost")
(def ^:const default-port 6062)
(def ^:const default-timeout-ms 15000)

(defn client
  "Build a Rockbox client.

  Options (all optional):

    :host         hostname or IP of rockboxd        (default \"localhost\")
    :port         GraphQL HTTP/WS port              (default 6062)
    :http-url     full HTTP URL override            (overrides host/port)
    :ws-url       full WebSocket URL override       (overrides host/port)
    :timeout-ms   request timeout in milliseconds   (default 15000)
    :headers      map of extra HTTP headers
    :http-client  java.net.http.HttpClient instance to reuse

  The returned value is a plain map — passable to all `rockbox.*` functions
  as their first argument so calls compose with `->`.

      (def c (rb/client))
      (def c (rb/client {:host \"music.home\" :port 6062}))
      (def c (rb/client {:http-url \"https://music.home/graphql\"}))"
  ([] (client {}))
  ([{:keys [host port http-url ws-url timeout-ms headers http-client]}]
   (let [host (or host default-host)
         port (or port default-port)
         http (or http-url (str "http://" host ":" port "/graphql"))
         wss  (or ws-url   (str "ws://"   host ":" port "/graphql"))]
     {:host        host
      :port        port
      :http-url    http
      :ws-url      wss
      :timeout-ms  (or timeout-ms default-timeout-ms)
      :headers     (or headers {})
      :http-client http-client
      :listeners   (atom {})
      :plugins     (atom {})
      :ws-conn     (atom nil)})))

;; --- builder-style helpers (composable via `->`) ----------------------------

(defn- rebuild-urls [c]
  (assoc c
         :http-url (str "http://" (:host c) ":" (:port c) "/graphql")
         :ws-url   (str "ws://"   (:host c) ":" (:port c) "/graphql")))

(defn with-host
  "Return `c` with `:host` set to `h` (and URLs rebuilt). Pipe-friendly."
  [c h] (rebuild-urls (assoc c :host h)))

(defn with-port  [c p]  (rebuild-urls (assoc c :port p)))
(defn with-timeout
  "Return `c` with `:timeout-ms` set. Pipe-friendly."
  [c ms] (assoc c :timeout-ms ms))
(defn with-headers
  "Merge extra headers into `c`. Pipe-friendly."
  [c h] (update c :headers merge h))
(defn with-http-url [c url] (assoc c :http-url url))
(defn with-ws-url   [c url] (assoc c :ws-url   url))

;; ---------------------------------------------------------------------------
;; Real-time subscriptions
;; ---------------------------------------------------------------------------

(def ^:private currently-playing-query
  "subscription CurrentlyPlaying {
     currentlyPlayingSong {
       id title artist album albumArt albumId artistId path length elapsed
     }
   }")

(def ^:private playback-status-query
  "subscription PlaybackStatus { playbackStatus { status } }")

(def ^:private playlist-changed-query
  "subscription PlaylistChanged {
     playlistChanged {
       amount index maxPlaylistSize firstIndex lastInsertPos seed lastShuffledStart
       tracks { id title artist album path length albumArt }
     }
   }")

(defn connect
  "Open the WebSocket and start the three default subscriptions
  (track / status / playlist). Idempotent. Returns the client."
  [client]
  (when-not @(:ws-conn client)
    (let [conn (ws/open client (:ws-url client))]
      (reset! (:ws-conn client) conn)
      (ws/subscribe conn currently-playing-query nil
                    {:next     (fn [{:keys [data]}]
                                 (when-let [track (:currently-playing-song data)]
                                   (events/emit client :track-changed track)))
                     :error    (fn [e] (events/emit client :ws-error e))
                     :complete (fn [])})
      (ws/subscribe conn playback-status-query nil
                    {:next     (fn [{:keys [data]}]
                                 (when-let [s (get-in data [:playback-status :status])]
                                   (events/emit client :status-changed s)))
                     :error    (fn [e] (events/emit client :ws-error e))
                     :complete (fn [])})
      (ws/subscribe conn playlist-changed-query nil
                    {:next     (fn [{:keys [data]}]
                                 (when-let [pl (:playlist-changed data)]
                                   (events/emit client :playlist-changed pl)))
                     :error    (fn [e] (events/emit client :ws-error e))
                     :complete (fn [])})))
  client)

(defn disconnect
  "Close the WebSocket connection. Returns the client."
  [client]
  (when-let [conn @(:ws-conn client)]
    (ws/close conn)
    (reset! (:ws-conn client) nil))
  client)

;; ---------------------------------------------------------------------------
;; Event API — re-exported from rockbox.events for convenience
;; ---------------------------------------------------------------------------

(def ^{:doc "See `rockbox.events/on`."}        on        events/on)
(def ^{:doc "See `rockbox.events/once`."}      once      events/once)
(def ^{:doc "See `rockbox.events/off`."}       off       events/off)
(def ^{:doc "See `rockbox.events/off-all`."}   off-all   events/off-all)
(def ^{:doc "See `rockbox.events/channel`."}   channel   events/channel)

;; ---------------------------------------------------------------------------
;; Plugin API — re-exported
;; ---------------------------------------------------------------------------

(defn use-plugin
  "Install a plugin. See `rockbox.plugin`. Returns the client."
  [client plugin]
  (plugin/install client plugin))

(defn unuse-plugin
  "Uninstall a plugin by name. Returns the client."
  [client plugin-name]
  (plugin/uninstall client plugin-name))

(defn installed-plugins
  "List installed plugins."
  [client]
  (plugin/installed client))

;; ---------------------------------------------------------------------------
;; Raw GraphQL escape hatch
;; ---------------------------------------------------------------------------

(defn query
  "Execute a raw GraphQL query/mutation. Returns the kebabized `data` map.
  Use this for operations not yet covered by the SDK.

      (rb/query client \"query { rockboxVersion }\")
      ;=> {:rockbox-version \"1.0.0\"}

      (rb/query client
                \"query Album($id: String!) { album(id: $id) { id title } }\"
                {:id \"abc-123\"})"
  ([client gql]      (transport/execute client gql))
  ([client gql vars] (transport/execute client gql vars)))
