(ns ex05-realtime-events
  "Open the WebSocket and react to track / status changes for 60 seconds."
  (:require [example-client :as client]
            [rockbox.core  :as rb]
            [rockbox.types :as t]))

(defn -main [& _]
  (let [c (client/make-client)]
    (-> c
        (rb/connect)
        (rb/on :track-changed
               (fn [tr]
                 (printf "▶ %s — %s%n" (:title tr) (:artist tr))))
        (rb/on :status-changed
               (fn [raw]
                 (printf "  status: %s%n" (t/playback-status->keyword raw))))
        (rb/on :playlist-changed
               (fn [pl]
                 (printf "  queue updated: %d tracks%n" (:amount pl))))
        (rb/on :ws-error
               (fn [e]
                 (println "WS error:" (or (.getMessage ^Throwable e) e)))))

    (println "Listening for 60 s — press Ctrl-C to quit early.")
    (Thread/sleep 60000)
    (rb/disconnect c)))
