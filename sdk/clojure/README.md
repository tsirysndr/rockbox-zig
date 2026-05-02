# rockbox-clj

Idiomatic Clojure SDK for [Rockbox Zig](https://github.com/tsirysndr/rockbox-zig) — a thin,
zero-dependency-heavy wrapper around rockboxd's GraphQL API with real-time
WebSocket subscriptions and a tiny plugin system.

* **Pipe-friendly.** Every function takes the client as its first argument.
  Action functions return the client so they compose with `->`.
* **Builder-friendly.** `with-host`, `with-port`, `with-timeout`,
  `with-headers`, `with-http-url`, `with-ws-url` — all pure, all chainable.
* **Clojure-friendly.** Plain maps with kebab-case keys both in and out;
  enums exposed as keywords; events surface as callbacks _or_ `core.async`
  channels; plugins are plain maps you `assoc` into shape.
* **Light dependencies.** Only `org.clojure/data.json` and `core.async` —
  HTTP and WebSockets ride on JDK 11+'s built-in `java.net.http`.

---

## Table of contents

- [Installation](#installation)
- [Quick start](#quick-start)
- [Configuration](#configuration)
- [API reference](#api-reference)
  - [Playback](#playback)
  - [Library](#library)
  - [Playlist (queue)](#playlist-queue)
  - [Saved playlists](#saved-playlists)
  - [Smart playlists](#smart-playlists)
  - [Sound](#sound)
  - [Settings](#settings)
  - [System](#system)
  - [Browse (filesystem)](#browse-filesystem)
  - [Devices](#devices)
  - [Bluetooth](#bluetooth)
- [Real-time events](#real-time-events)
- [Plugin system](#plugin-system)
- [Error handling](#error-handling)
- [Raw GraphQL queries](#raw-graphql-queries)
- [Types reference](#types-reference)

---

## Installation

`deps.edn`:

```clojure
{:deps {org.clojars.tsiry/rockbox-clj {:git/url "https://github.com/tsirysndr/rockbox-zig"
                                       :git/sha "..."
                                       :deps/root "sdk/clojure"}}}
```

Or pin via local path while developing:

```clojure
{:deps {org.clojars.tsiry/rockbox-clj {:local/root "/path/to/rockbox-zig/sdk/clojure"}}}
```

### Publishing to Clojars (maintainers)

```sh
cd sdk/clojure

# Bump version, then build the JAR
VERSION=0.1.0 clojure -T:build jar

# Install to local ~/.m2 for testing
VERSION=0.1.0 clojure -T:build install

# Deploy to Clojars (set creds first)
export CLOJARS_USERNAME=tsiry
export CLOJARS_PASSWORD=<deploy-token>
VERSION=0.1.0 clojure -T:build deploy
```

`rockboxd` must be running and reachable. By default the SDK connects to
`http://localhost:6062/graphql`. Start it with:

```sh
rockbox start
```

---

## Quick start

```clojure
(require '[rockbox.core    :as rb]
         '[rockbox.playback :as pb]
         '[rockbox.library  :as lib])

(def client (rb/client))

;; Optional: open the WebSocket so subscribers start receiving events
(rb/connect client)

;; What's playing right now?
(when-let [t (pb/current-track client)]
  (println "Now playing:" (:title t) "—" (:artist t)))

;; Search the library
(let [{:keys [albums tracks]} (lib/search client "dark side")]
  (println (count albums) "albums," (count tracks) "tracks"))

;; Play an album, shuffled — in one piped chain
(-> client
    (pb/play-album "album-id" {:shuffle true}))

;; React to track changes
(rb/on client :track-changed
  (fn [t] (println "▶" (:title t) "by" (:artist t))))

;; Tear down when done
(rb/disconnect client)
```

---

## Configuration

```clojure
(require '[rockbox.core :as rb])

;; Defaults: localhost:6062
(def c (rb/client))

;; Custom host and port
(def c (rb/client {:host "192.168.1.42" :port 6062}))

;; Fully custom URLs (e.g. behind a reverse proxy)
(def c (rb/client {:http-url "https://music.home/graphql"
                   :ws-url   "wss://music.home/graphql"}))

;; Builder style — every with-* fn returns a new client value
(def c (-> (rb/client)
           (rb/with-host    "music.home")
           (rb/with-port    6062)
           (rb/with-timeout 30000)
           (rb/with-headers {:x-trace-id "req-123"})))
```

| Option        | Default                           | Description                         |
|---------------|-----------------------------------|-------------------------------------|
| `:host`       | `"localhost"`                     | rockboxd hostname / IP              |
| `:port`       | `6062`                            | GraphQL HTTP/WS port                |
| `:http-url`   | `http://{host}:{port}/graphql`    | Override the full HTTP URL          |
| `:ws-url`     | `ws://{host}:{port}/graphql`      | Override the full WS URL            |
| `:timeout-ms` | `15000`                           | Per-request timeout                 |
| `:headers`    | `{}`                              | Extra HTTP headers map              |
| `:http-client`| (auto)                            | Reuse a `java.net.http.HttpClient`  |

---

## API reference

> Convention: **action functions return the client** so chains compose with
> `->`. **Read functions return data** as plain Clojure maps with kebab-case
> keys.

### Playback

```clojure
(require '[rockbox.playback :as pb]
         '[rockbox.types    :as t])

;; Status
(pb/status client)        ;=> :playing | :paused | :stopped
(pb/raw-status client)    ;=> 0 | 1 | 3   (raw firmware enum)

;; Current / next track
(pb/current-track client) ;=> {:title "..." :artist "..." :elapsed 12345 ...} or nil
(pb/next-track    client)
(pb/file-position client)

;; Transport — pipe-friendly
(-> client
    (pb/pause)
    (pb/seek 90000)       ; jump to 1:30 (ms)
    (pb/resume))

(pb/play     client)
(pb/play     client {:elapsed 0 :offset 0})
(pb/next     client)
(pb/previous client)
(pb/stop     client)
(pb/flush-and-reload client)

;; Single-call play helpers
(pb/play-track     client "/Music/Pink Floyd/Wish You Were Here.mp3")
(pb/play-album     client "album-id" {:shuffle true})
(pb/play-album     client "album-id" {:position 3})
(pb/play-artist    client "artist-id" {:shuffle true})
(pb/play-playlist  client "playlist-id" {:shuffle true})
(pb/play-directory client "/Music/Jazz" {:recurse true :shuffle true})
(pb/play-liked-tracks client {:shuffle true})
(pb/play-all-tracks   client {:shuffle true})
```

---

### Library

```clojure
(require '[rockbox.library :as lib])

;; Albums
(lib/albums       client)             ;=> vector of album maps with shallow track stubs
(lib/album        client "album-id")  ;=> album with full track list, or nil
(lib/liked-albums client)
(lib/like-album   client "album-id")
(lib/unlike-album client "album-id")

;; Artists
(lib/artists client)
(lib/artist  client "artist-id")

;; Tracks
(lib/tracks       client)
(lib/track        client "track-id")
(lib/liked-tracks client)
(lib/like-track   client "track-id")
(lib/unlike-track client "track-id")

;; Search — returns {:artists :albums :tracks :liked-tracks :liked-albums}
(let [{:keys [albums tracks]} (lib/search client "radiohead")]
  (println (count albums) "albums," (count tracks) "tracks"))

;; Trigger a full library scan
(lib/scan client)
```

---

### Playlist (queue)

The *playlist* namespace manages the live playback queue. For persistent
named collections use [Saved playlists](#saved-playlists).

```clojure
(require '[rockbox.playlist :as q]
         '[rockbox.types    :as t])

;; Inspect
(q/current client)   ;=> {:tracks [...] :amount n :index i ...}
(q/amount  client)

;; Queue management — every mutation returns the client
(-> client
    (q/insert-tracks ["/Music/a.mp3" "/Music/b.mp3"] :next)
    (q/insert-album  "album-id" :last)
    (q/shuffle))

(q/insert-directory client "/Music/Ambient" :last)
(q/remove-track     client 2)            ; remove queue index 2
(q/clear            client)
(q/create           client "Evening Mix" ["/Music/a.mp3" "/Music/b.mp3"])
(q/start            client {:start-index 0})
(q/resume           client)
```

| `insert-position` keyword | Effect                                 |
|---------------------------|----------------------------------------|
| `:next`                   | After the currently playing track      |
| `:after-current`          | After the last manually inserted track |
| `:last`                   | At the end of the queue                |
| `:first`                  | Replace the entire queue               |

(You can also pass the underlying integer if you prefer.)

---

### Saved playlists

```clojure
(require '[rockbox.saved-playlists :as sp])

(sp/list       client)              ; all
(sp/list       client "folder-id")  ; in a folder
(sp/get        client "playlist-id")
(sp/track-ids  client "playlist-id")

(sp/create client {:name        "Late Night Jazz"
                   :description "Quiet music for working"
                   :folder-id   "folder-id"
                   :track-ids   ["t1" "t2" "t3"]})

(sp/update client "playlist-id" {:name "Late Night Jazz (updated)"})

(sp/add-tracks   client "playlist-id" ["t4" "t5"])
(sp/remove-track client "playlist-id" "t1")
(sp/play         client "playlist-id")
(sp/delete       client "playlist-id")

;; Folders
(sp/folders       client)
(sp/create-folder client "Work")
(sp/delete-folder client "folder-id")
```

---

### Smart playlists

Smart playlists evaluate a rule set dynamically. The SDK accepts the
`:rules` value as either a JSON string or any Clojure data structure (it
will JSON-encode for you).

```clojure
(require '[rockbox.smart-playlists :as smart])

(smart/list      client)
(smart/get       client "smart-id")
(smart/track-ids client "smart-id")     ; resolve to matching track ids

;; Create — rules as plain Clojure data
(smart/create client
  {:name  "Recently played"
   :rules {:operator "AND"
           :rules    [{:field "play_count"  :op "gt"     :value 0}
                      {:field "last_played" :op "within" :value "30d"}]}})

;; Or as a pre-baked JSON string
(smart/create client {:name "Top 50" :rules "{\"sort\":{...}}"})

(smart/update client "smart-id" {:name "Recently played (60d)"
                                 :rules {...}})
(smart/play   client "smart-id")
(smart/delete client "smart-id")

;; Listening stats — feeds smart-playlist rules and scrobblers
(smart/track-stats     client "track-id") ;=> {:play-count n :skip-count n :last-played t}
(smart/record-played   client "track-id")
(smart/record-skipped  client "track-id")
```

---

### Sound

Volume is measured in firmware-defined steps (not absolute dB). The number
of steps per dB varies by hardware target.

```clojure
(require '[rockbox.sound :as snd])

(snd/volume        client)     ;=> {:volume v :min m :max M}
(snd/adjust-volume client +3)  ; 3 steps up;  returns the new raw volume
(snd/volume-up     client)     ; +1
(snd/volume-down   client)     ; -1
```

---

### Settings

```clojure
(require '[rockbox.settings :as settings])

(def s (settings/get client))
(println :music-dir   (:music-dir   s)
         :volume      (:volume      s)
         :eq-enabled  (:eq-enabled  s)
         :repeat-mode (:repeat-mode s))

;; Partial update — only the keys you pass are written
(settings/save client
               {:shuffle     true
                :repeat-mode 1})    ; or use rockbox.types/repeat-mode

;; Enable a 5-band EQ
(settings/save client
               {:eq-enabled       true
                :eq-precut        -3
                :eq-band-settings [{:cutoff   60 :q 7 :gain  3}
                                   {:cutoff  200 :q 7 :gain  0}
                                   {:cutoff  800 :q 7 :gain  0}
                                   {:cutoff 4000 :q 7 :gain -2}
                                   {:cutoff 12000 :q 7 :gain  1}]})

;; Compressor + ReplayGain
(settings/save client
               {:compressor-settings {:threshold -24 :makeup-gain 3
                                      :ratio 2     :knee 0
                                      :attack-time 5 :release-time 100}
                :replaygain-settings {:noclip true :type 1 :preamp 0}})
```

---

### System

```clojure
(require '[rockbox.system :as sys])

(sys/version client)   ;=> "1.0.0"
(sys/status  client)   ;=> {:runtime n :topruntime n :resume-index i ...}
```

---

### Browse (filesystem)

Walk the configured `music_dir`.

```clojure
(require '[rockbox.browse :as br]
         '[rockbox.types  :as t])

(br/entries     client)                       ; root of music_dir
(br/entries     client "/Music/Pink Floyd")
(br/directories client "/Music")
(br/files       client "/Music/Pink Floyd/The Wall")

;; Or filter manually
(filter t/directory? (br/entries client))
```

---

### Devices

Output sinks discovered via mDNS — Chromecast, AirPlay, etc.

```clojure
(require '[rockbox.devices :as dev])

(dev/list       client)
(dev/get        client "device-id")
(dev/connect    client "chromecast-id")     ; switches the active PCM sink
(dev/disconnect client "chromecast-id")     ; reverts to built-in
```

---

### Bluetooth

Linux-only (BlueZ via D-Bus).

```clojure
(require '[rockbox.bluetooth :as bt])

(bt/devices    client)
(bt/scan       client)         ; default timeout
(bt/scan       client 30)      ; 30 s
(bt/connect    client "AA:BB:CC:DD:EE:FF")
(bt/disconnect client "AA:BB:CC:DD:EE:FF")
```

---

## Real-time events

Call `(rb/connect client)` to open the WebSocket. The connection is lazy
(only created on first call), auto-reconnects with exponential backoff up
to 30 s, and re-subscribes after every reconnect.

```clojure
(require '[rockbox.core   :as rb]
         '[rockbox.types  :as t])

(rb/connect client)

;; ── Callback API ────────────────────────────────────────────────────────────
(-> client
    (rb/on :track-changed
           (fn [tr] (println "▶" (:title tr) "—" (:artist tr))))
    (rb/on :status-changed
           (fn [raw] (println "status:" (t/playback-status->keyword raw))))
    (rb/on :playlist-changed
           (fn [pl] (println "queue updated:" (:amount pl) "tracks")))
    (rb/on :ws-error
           (fn [e] (println "WS error:" (.getMessage ^Throwable e)))))

;; One-shot listener
(rb/once client :track-changed (fn [tr] (println "First track:" (:title tr))))

;; Remove a listener
(let [h (fn [tr] (println (:title tr)))]
  (rb/on client :track-changed h)
  ;; …later
  (rb/off client :track-changed h))

;; ── core.async API ──────────────────────────────────────────────────────────
(require '[clojure.core.async :as a]
         '[rockbox.events     :as events])

(let [ch (events/channel client :track-changed)]
  (a/go-loop []
    (when-let [tr (a/<! ch)]
      (println "▶" (:title tr))
      (recur)))
  ;; …later
  (events/close-channel! client ch))

;; Shut everything down
(rb/disconnect client)
```

### Event map

| Event               | Payload     | Description                          |
|---------------------|-------------|--------------------------------------|
| `:track-changed`    | track map   | Currently playing track changed      |
| `:status-changed`   | int         | Playback status (0=stopped, 1=playing, 3=paused) |
| `:playlist-changed` | playlist    | Active queue was modified            |
| `:ws-open`          | `nil`       | WebSocket connection established     |
| `:ws-close`         | `nil`       | WebSocket connection closed          |
| `:ws-error`         | Throwable   | WebSocket / subscription error       |

---

## Plugin system

A plugin is a plain map with `:name`, `:install`, and (optionally) `:version`,
`:description`, and `:uninstall`. Compose them with `assoc` / closures.

```clojure
(defn lastfm-scrobbler [{:keys [api-key secret]}]
  (let [state (atom {:current nil :started-at 0})]
    {:name        "lastfm-scrobbler"
     :version     "1.0.0"
     :description "Scrobble plays > 30 s old to Last.fm"
     :install
     (fn [{:keys [client query events]}]
       ;; `events` is a map of helpers already partially-applied to `client`
       ((:on events) :track-changed
        (fn [tr]
          (let [{:keys [current started-at]} @state]
            (when (and current (> (- (System/currentTimeMillis) started-at) 30000))
              (submit-to-lastfm api-key secret current))
            (reset! state {:current tr :started-at (System/currentTimeMillis)})))))
     :uninstall   (fn [] (reset! state {}))}))

(rb/use-plugin client (lastfm-scrobbler {:api-key "..." :secret "..."}))
(rb/installed-plugins client)              ;=> [{:name "lastfm-scrobbler" ...}]
(rb/unuse-plugin     client "lastfm-scrobbler")
```

The `install` fn receives a context map:

```clojure
{:client client                         ; the client value
 :query  (fn ([gql] ...) ([gql vars] ...))
 :events {:on            (partial events/on            client)
          :once          (partial events/once          client)
          :off           (partial events/off           client)
          :off-all       (partial events/off-all       client)
          :channel       (partial events/channel       client)
          :close-channel (partial events/close-channel! client)}}
```

### Plugin with custom queries

```clojure
(def lyrics-plugin
  {:name    "lyrics"
   :version "0.1.0"
   :install (fn [{:keys [query events]}]
              ((:on events) :track-changed
               (fn [tr]
                 (when (:id tr)
                   (let [data (query "query T($id: String!) { track(id: $id) { title artist } }"
                                     {:id (:id tr)})]
                     (fetch-and-display-lyrics (:track data)))))))})
```

### Sleep timer plugin (closes over local state)

```clojure
(defn sleep-timer [minutes]
  (let [t (atom nil)]
    {:name        "sleep-timer"
     :version     "1.0.0"
     :description (str "Stop playback after " minutes " minutes")
     :install
     (fn [{:keys [query events]}]
       (reset! t (future
                   (Thread/sleep (* minutes 60 1000))
                   (query "mutation { hardStop }")
                   (println "Sleep timer fired — playback stopped.")))
       ((:on events) :status-changed
        (fn [s] (when (zero? s) (some-> @t future-cancel)))))
     :uninstall   (fn [] (some-> @t future-cancel))}))

(rb/use-plugin client (sleep-timer 30))
```

---

## Error handling

All errors are `clojure.lang.ExceptionInfo` instances carrying a `:type` key
in their ex-data. One `catch ExceptionInfo` covers everything:

```clojure
(require '[rockbox.errors :as err])

(try
  (pb/play client)
  (catch clojure.lang.ExceptionInfo e
    (case (:type (ex-data e))
      :rockbox/network (println "rockboxd is offline:" (.getMessage e))
      :rockbox/graphql (doseq [g (:errors (ex-data e))]
                         (println "GraphQL:" (:message g) (:path g)))
      :rockbox/config  (println "Bad input:" (.getMessage e))
      (throw e))))

;; Predicates
(err/network-error? e)
(err/graphql-error? e)
```

| `:type`            | When thrown                                                     |
|--------------------|-----------------------------------------------------------------|
| `:rockbox/network` | Cannot reach rockboxd, or HTTP returned a non-2xx status        |
| `:rockbox/graphql` | Server returned `{errors: [...]}` in the response body          |
| `:rockbox/config`  | Client constructed with bad config or required input missing    |

---

## Raw GraphQL queries

For operations not yet covered by the SDK, use `rb/query`. The GraphiQL
explorer is available at `http://localhost:6062/graphiql` while rockboxd
is running.

```clojure
;; Simple query
(rb/query client "query { rockboxVersion }")
;=> {:rockbox-version "1.0.0"}

;; With variables — kebab-case is auto-converted to camelCase
(rb/query client
          "query Album($id: String!) {
             album(id: $id) { id title artist year }
           }"
          {:id "abc-123"})

;; Mutation
(rb/query client
          "mutation Seek($t: Int!) { fastForwardRewind(newTime: $t) }"
          {:t 120000})
```

---

## Types reference

Enum constants and helpers live in `rockbox.types`:

```clojure
(require '[rockbox.types :as t])

t/playback-status              ;=> {:stopped 0, :playing 1, :paused 3}
t/playback-status->keyword     ;=> {0 :stopped, 1 :playing, 3 :paused}
t/playing                      ;=> 1
t/repeat-mode                  ;=> {:off 0, :all 1, :one 2, :shuffle 3, :ab-repeat 4}
t/channel-config               ;=> {:stereo 0, :stereo-narrow 1, ...}
t/replaygain-type              ;=> {:track 0, :album 1, :shuffle 2}
t/insert-position              ;=> {:next 0, :after-current 1, :last 2, :first 3}

(t/->insert-position :next)    ; coerce keyword or int -> int
(t/directory? entry)           ; tests entry's :attr bitmask
(t/file?      entry)
(t/format-ms  75000)           ;=> "1:15"
```

### Selected response shapes

`Track` (kebab-case keys):

```clojure
{:id "..."   :title "..." :artist "..." :album "..."
 :genre "..." :album-artist "..." :composer "..."
 :tracknum 1 :discnum 1 :year 1973
 :bitrate 320 :frequency 44100
 :length 12345  ; ms
 :elapsed 6789  ; ms
 :filesize 4567890 :path "/Music/..."
 :album-id "..."  :artist-id "..."  :album-art "..."}
```

`Playlist`:

```clojure
{:amount 12 :index 3 :max-playlist-size 32000
 :first-index 0 :last-insert-pos -1
 :seed 0 :last-shuffled-start 0
 :tracks [...]}
```

`Device`:

```clojure
{:id "..." :name "..." :host "..." :ip "..." :port 8009
 :service "..." :app "..." :base-url "..."
 :is-connected     false
 :is-cast-device   true
 :is-source-device false
 :is-current-device false}
```

---

## License

MIT License. See [LICENSE](./LICENSE) for details.
