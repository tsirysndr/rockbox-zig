(ns rockbox.ws
  "WebSocket transport implementing the `graphql-ws` subprotocol
  (https://github.com/enisdenjo/graphql-ws/blob/master/PROTOCOL.md).

  Uses `java.net.http.WebSocket` so there is no third-party WS dependency.
  Auto-reconnects with exponential backoff up to 30 s. Fires `:ws-open`,
  `:ws-close`, `:ws-error` events through the SDK's event registry.

  This is an internal namespace — call `rockbox.core/connect` instead."
  (:require [clojure.data.json :as json]
            [rockbox.events :as events]
            [rockbox.util   :as util])
  (:import (java.net URI)
           (java.net.http HttpClient WebSocket WebSocket$Builder
                          WebSocket$Listener)
           (java.time Duration)
           (java.util.concurrent CompletableFuture
                                 ConcurrentHashMap)
           (java.util.concurrent.atomic AtomicLong AtomicBoolean)))

;; ---------------------------------------------------------------------------
;; Connection record
;; ---------------------------------------------------------------------------
;; Keys:
;;   :url           — ws://host:port/graphql
;;   :client        — the rockbox client value (for emitting events)
;;   :ws            — atom holding the current WebSocket instance
;;   :alive?        — AtomicBoolean — true while the user wants the conn open
;;   :next-id       — AtomicLong, monotonic op id
;;   :subscriptions — ConcurrentHashMap<id, {:query :variables :sink}>
;;                    `sink` is `{:next fn :error fn :complete fn}`
;;   :ack?          — atom: has the server sent connection_ack?
;;   :buf           — atom (StringBuilder) for assembling fragmented frames

(defn- new-connection [url client]
  {:url           url
   :client        client
   :ws            (atom nil)
   :alive?        (AtomicBoolean. true)
   :next-id       (AtomicLong. 1)
   :subscriptions (ConcurrentHashMap.)
   :ack?          (atom false)
   :buf           (atom (StringBuilder.))})

;; ---------------------------------------------------------------------------
;; Sending
;; ---------------------------------------------------------------------------

(defn- send-text! [conn msg]
  (when-let [^WebSocket ws @(:ws conn)]
    (try
      (.sendText ws (json/write-str msg) true)
      (catch Throwable t
        (events/emit (:client conn) :ws-error t)))))

(declare connect!)

(defn- emit-buffered-subscriptions! [conn]
  (doseq [[id sub] (.entrySet ^ConcurrentHashMap (:subscriptions conn))]
    (let [k (.getKey ^java.util.Map$Entry id)
          v (.getValue ^java.util.Map$Entry id)]
      ;; Re-send subscribe frames after reconnect
      (send-text! conn
                  {"id" k
                   "type" "subscribe"
                   "payload"
                   (cond-> {"query" (:query v)}
                     (some? (:variables v))
                     (assoc "variables" (util/camelize-keys (:variables v))))}))))

;; ---------------------------------------------------------------------------
;; Listener — handles all incoming frames + lifecycle events
;; ---------------------------------------------------------------------------

(defn- handle-message [conn ^String text]
  (let [msg     (try (json/read-str text) (catch Exception _ nil))
        msg-type (get msg "type")]
    (case msg-type
      "connection_ack"
      (do (reset! (:ack? conn) true)
          (events/emit (:client conn) :ws-open nil)
          (emit-buffered-subscriptions! conn))

      "ping"
      (send-text! conn {"type" "pong"})

      "pong"  nil

      ("next" "data")
      (let [id   (get msg "id")
            data (some-> (get-in msg ["payload" "data"]) util/kebabize-keys)
            sink (some-> ^ConcurrentHashMap (:subscriptions conn) (.get id) :sink)]
        (when sink ((:next sink) {:data data})))

      "error"
      (let [id   (get msg "id")
            errs (get-in msg ["payload"])
            sink (some-> ^ConcurrentHashMap (:subscriptions conn) (.get id) :sink)]
        (when sink ((:error sink) errs)))

      "complete"
      (let [id   (get msg "id")
            sink (some-> ^ConcurrentHashMap (:subscriptions conn) (.get id) :sink)]
        (when sink ((:complete sink)))
        (.remove ^ConcurrentHashMap (:subscriptions conn) id))

      ;; Ignore anything we don't recognise
      nil)))

(defn- backoff-ms [attempt]
  (min 30000 (long (* 1000 (Math/pow 2 (min attempt 10))))))

(defn- schedule-reconnect [conn attempt]
  (when (.get ^AtomicBoolean (:alive? conn))
    (let [delay (backoff-ms attempt)]
      (future
        (Thread/sleep delay)
        (when (.get ^AtomicBoolean (:alive? conn))
          (try (connect! conn (inc attempt))
               (catch Throwable t
                 (events/emit (:client conn) :ws-error t)
                 (schedule-reconnect conn (inc attempt)))))))))

(defn- ^WebSocket$Listener make-listener [conn ^"[I" reconnect-attempt]
  (reify WebSocket$Listener
    (onOpen [_ ws]
      (.request ^WebSocket ws 1)
      (aset reconnect-attempt 0 0)
      ;; graphql-ws handshake
      (send-text! conn {"type" "connection_init"}))

    (onText [_ ws data last?]
      (let [^StringBuilder sb @(:buf conn)]
        (.append sb (str data))
        (when last?
          (let [text (.toString sb)]
            (reset! (:buf conn) (StringBuilder.))
            (handle-message conn text))))
      (.request ^WebSocket ws 1)
      nil)

    (onBinary [_ ws _data _last?]
      (.request ^WebSocket ws 1)
      nil)

    (onPing [_ ws msg]
      (.sendPong ^WebSocket ws msg))

    (onPong [_ ws _msg]
      (.request ^WebSocket ws 1)
      nil)

    (onClose [_ _ws _code _reason]
      (reset! (:ack? conn) false)
      (reset! (:ws   conn) nil)
      (events/emit (:client conn) :ws-close nil)
      (when (.get ^AtomicBoolean (:alive? conn))
        (schedule-reconnect conn (aget reconnect-attempt 0)))
      nil)

    (onError [_ _ws err]
      (reset! (:ack? conn) false)
      (events/emit (:client conn) :ws-error err)
      (aset reconnect-attempt 0 (inc (aget reconnect-attempt 0)))
      (when (.get ^AtomicBoolean (:alive? conn))
        (schedule-reconnect conn (aget reconnect-attempt 0))))))

;; ---------------------------------------------------------------------------
;; Connect / disconnect
;; ---------------------------------------------------------------------------

(defn- connect!
  ([conn] (connect! conn 0))
  ([conn attempt]
   (let [http (HttpClient/newHttpClient)
         attempts-arr (int-array 1 attempt)
         listener (make-listener conn attempts-arr)
         ^CompletableFuture cf
         (-> (.newWebSocketBuilder http)
             (.subprotocols "graphql-transport-ws" (into-array String []))
             (.connectTimeout (Duration/ofSeconds 15))
             (.buildAsync (URI/create (:url conn)) listener))]
     (-> cf
         (.thenAccept (reify java.util.function.Consumer
                        (accept [_ ws]
                          (reset! (:ws conn) ws))))
         (.exceptionally (reify java.util.function.Function
                           (apply [_ throwable]
                             (events/emit (:client conn) :ws-error throwable)
                             (schedule-reconnect conn (inc attempt))
                             nil)))))
   conn))

(defn open
  "Open a new WebSocket connection. Returns a `connection` value that you
  pass to `subscribe` and `close`."
  [client ws-url]
  (let [conn (new-connection ws-url client)]
    (connect! conn 0)
    conn))

(defn close
  "Tear down the WebSocket connection. Idempotent."
  [conn]
  (when conn
    (.set ^AtomicBoolean (:alive? conn) false)
    (when-let [^WebSocket ws @(:ws conn)]
      (try (.sendClose ws WebSocket/NORMAL_CLOSURE "bye")
           (catch Throwable _ nil)))
    (reset! (:ws conn) nil)
    (.clear ^ConcurrentHashMap (:subscriptions conn)))
  nil)

;; ---------------------------------------------------------------------------
;; Subscriptions
;; ---------------------------------------------------------------------------

(defn subscribe
  "Start a GraphQL subscription. `sink` is a map with `:next`, `:error`,
  `:complete` keys. Returns a 0-arity unsubscribe fn."
  [conn query variables sink]
  (let [id   (str (.getAndIncrement ^AtomicLong (:next-id conn)))
        full {:query query :variables variables :sink sink}]
    (.put ^ConcurrentHashMap (:subscriptions conn) id full)
    (when @(:ack? conn)
      (send-text! conn
                  {"id"   id
                   "type" "subscribe"
                   "payload"
                   (cond-> {"query" query}
                     (some? variables)
                     (assoc "variables" (util/camelize-keys variables)))}))
    (fn unsubscribe []
      (when (.remove ^ConcurrentHashMap (:subscriptions conn) id)
        (send-text! conn {"id" id "type" "complete"})))))
