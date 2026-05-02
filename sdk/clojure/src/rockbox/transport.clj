(ns rockbox.transport
  "HTTP transport for GraphQL queries — built on `java.net.http.HttpClient`
  so the SDK has no third-party HTTP dependency.

  You normally don't call this directly; use `rockbox.core/query` or one of
  the domain APIs (`rockbox.playback`, `rockbox.library`, ...). Exposed for
  plugin authors and advanced consumers who need a stable hook."
  (:require [clojure.data.json :as json]
            [rockbox.errors :as err]
            [rockbox.util   :as util])
  (:import (java.net URI)
           (java.net.http HttpClient HttpClient$Redirect
                          HttpRequest HttpResponse$BodyHandlers
                          HttpRequest$BodyPublishers)
           (java.time Duration)))

;; ---------------------------------------------------------------------------
;; HttpClient — one cached instance per JVM (thread-safe, reusable)
;; ---------------------------------------------------------------------------

(defonce ^:private default-client
  (delay
    (-> (HttpClient/newBuilder)
        (.connectTimeout (Duration/ofSeconds 10))
        (.followRedirects HttpClient$Redirect/NORMAL)
        (.build))))

(defn- http-client ^HttpClient [client]
  (or (:http-client client) @default-client))

;; ---------------------------------------------------------------------------
;; Request building
;; ---------------------------------------------------------------------------

(defn- ->headers ^"[Ljava.lang.String;" [headers]
  (let [base ["Content-Type" "application/json"
              "Accept"       "application/json"]
        all  (into base
                   (mapcat (fn [[k v]] [(name k) (str v)]))
                   headers)]
    (into-array String all)))

(defn- build-request ^HttpRequest [{:keys [http-url timeout-ms headers]} body-json]
  (let [b (-> (HttpRequest/newBuilder)
              (.uri (URI/create http-url))
              (.timeout (Duration/ofMillis (long (or timeout-ms 15000))))
              (.POST (HttpRequest$BodyPublishers/ofString body-json)))]
    (when (seq headers)
      (.headers b (->headers headers)))
    (.header b "Content-Type" "application/json")
    (.header b "Accept"       "application/json")
    (.build b)))

(defn- send-request [client req]
  (try
    (.send (http-client client) req (HttpResponse$BodyHandlers/ofString))
    (catch java.net.ConnectException e
      (throw (err/network-error
              (str "Failed to reach Rockbox at " (:http-url client)) e)))
    (catch java.net.http.HttpConnectTimeoutException e
      (throw (err/network-error "Connect timeout" e)))
    (catch java.net.http.HttpTimeoutException e
      (throw (err/network-error "HTTP timeout" e)))
    (catch java.io.IOException e
      (throw (err/network-error (.getMessage e) e)))))

;; ---------------------------------------------------------------------------
;; Public API
;; ---------------------------------------------------------------------------

(defn execute
  "Execute a GraphQL query/mutation and return the parsed `data` map (with
  kebab-case keyword keys).

  `variables` may be a Clojure map with kebab-case keys; they are converted to
  camelCase strings before sending. Returns the kebabized `data` payload, or
  throws an `ex-info` of type `:rockbox/network` or `:rockbox/graphql`."
  ([client query]           (execute client query nil))
  ([client query variables]
   (let [vars (some-> variables util/drop-nils not-empty util/camelize-keys)
         payload (cond-> {"query" query}
                   vars (assoc "variables" vars))
         body (json/write-str payload)
         req  (build-request client body)
         resp (send-request client req)
         status (.statusCode ^java.net.http.HttpResponse resp)
         body-str (.body ^java.net.http.HttpResponse resp)]
     (when-not (<= 200 status 299)
       (throw (err/network-error (str "HTTP " status " from " (:http-url client)))))
     (let [parsed (try
                    (json/read-str body-str)
                    (catch Exception e
                      (throw (err/network-error
                              (str "Invalid JSON response: " (.getMessage e)) e))))
           errors (get parsed "errors")]
       (when (and errors (seq errors))
         (throw (err/graphql-error (mapv util/kebabize-keys errors))))
       (-> (get parsed "data") util/kebabize-keys)))))

(defn execute-field
  "Convenience wrapper: execute the query, then pluck a single top-level
  field from the response. Most domain APIs use this for one-line bodies."
  ([client query field]
   (get (execute client query) field))
  ([client query variables field]
   (get (execute client query variables) field)))
