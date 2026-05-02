(ns rockbox.events
  "Event registry and dispatch.

  Listeners are stored on the client itself (in an atom) so a single client
  value can be passed around and shared between threads safely.

  ## Pipe-friendly callback API

      (-> client
          (rb/connect)
          (events/on :track-changed (fn [t] (println \"▶\" (:title t))))
          (events/on :status-changed (fn [s] (println \"status:\" s))))

  ## core.async channel API

      (require '[clojure.core.async :as a])
      (def ch (events/channel client :track-changed))
      (a/go-loop []
        (when-let [t (a/<! ch)]
          (println (:title t))
          (recur)))

  Supported events: `:track-changed :status-changed :playlist-changed
                    :ws-open :ws-close :ws-error`"
  (:require [clojure.core.async :as a]))

(def event-keys
  #{:track-changed :status-changed :playlist-changed
    :ws-open       :ws-close       :ws-error})

(defn- valid-event! [event]
  (when-not (contains? event-keys event)
    (throw (ex-info (str "Unknown event: " event ". Valid events: " event-keys)
                    {:type :rockbox/config :event event}))))

;; ---------------------------------------------------------------------------
;; Registry primitives — operate on the client's `:listeners` atom
;; ---------------------------------------------------------------------------

(defn- listeners-atom [client]
  (or (:listeners client)
      (throw (ex-info "Client has no listeners atom — was it built with rockbox.core/client?"
                      {:type :rockbox/config}))))

(defn on
  "Register a listener for an event. Returns the client (so the call composes
  with `->`). The listener receives one argument — the event payload (or `nil`
  for events with no payload, like `:ws-open`)."
  [client event listener]
  (valid-event! event)
  (swap! (listeners-atom client) update event (fnil conj #{}) listener)
  client)

(defn once
  "Register a one-shot listener. Returns the client."
  [client event listener]
  (valid-event! event)
  (let [registry (listeners-atom client)
        wrapped  (atom nil)]
    (reset! wrapped
            (fn [payload]
              (swap! registry update event disj @wrapped)
              (listener payload)))
    (swap! registry update event (fnil conj #{}) @wrapped)
    client))

(defn off
  "Remove a listener. Returns the client."
  [client event listener]
  (valid-event! event)
  (swap! (listeners-atom client) update event (fnil disj #{}) listener)
  client)

(defn off-all
  "Remove every listener (or every listener for a single event)."
  ([client]
   (reset! (listeners-atom client) {})
   client)
  ([client event]
   (valid-event! event)
   (swap! (listeners-atom client) dissoc event)
   client))

(defn emit
  "Internal — call every listener for `event`. Used by the WS layer."
  [client event payload]
  (doseq [f (get @(listeners-atom client) event)]
    (try (f payload)
         (catch Throwable t
           ;; Don't let one rogue listener break the others.
           (when-not (= event :ws-error)
             (doseq [g (get @(listeners-atom client) :ws-error)]
               (try (g t) (catch Throwable _ nil))))))))

;; ---------------------------------------------------------------------------
;; core.async bridge
;; ---------------------------------------------------------------------------

(defn channel
  "Return a `core.async` channel that receives every payload for `event`.
  The returned channel is closed when its underlying listener is removed via
  `close-channel!`. Buffer defaults to 16; pass `{:buf n}` (or a buffer) to
  override.

      (def ch (events/channel client :track-changed))
      (a/go-loop []
        (when-let [t (a/<! ch)]
          (println (:title t))
          (recur)))"
  ([client event] (channel client event {}))
  ([client event {:keys [buf] :or {buf 16}}]
   (valid-event! event)
   (let [ch (a/chan buf)
         f  (fn [payload] (a/put! ch payload))]
     (on client event f)
     ;; Stash the listener fn on the channel's metadata so close-channel! can
     ;; find it. core.async chans are not IObj, so use a side table.
     (swap! (listeners-atom client) update ::channels assoc ch [event f])
     ch)))

(defn close-channel!
  "Close a channel returned by `channel` and unregister its underlying listener."
  [client ch]
  (when-let [[event f] (get-in @(listeners-atom client) [::channels ch])]
    (off client event f)
    (swap! (listeners-atom client) update ::channels dissoc ch))
  (a/close! ch)
  client)
