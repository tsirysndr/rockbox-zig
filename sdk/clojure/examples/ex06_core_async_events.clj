(ns ex06-core-async-events
  "Same idea as 05, but consume events through a core.async channel —
  natural when integrating with a `go-loop`-based event pipeline."
  (:require [example-client :as client]
            [clojure.core.async :as a]
            [rockbox.core   :as rb]
            [rockbox.events :as events]))

(defn -main [& _]
  (let [c     (client/make-client)
        _     (rb/connect c)
        track-ch  (events/channel c :track-changed)
        status-ch (events/channel c :status-changed)]

    (a/go-loop []
      (when-let [tr (a/<! track-ch)]
        (printf "▶ %s — %s%n" (:title tr) (:artist tr))
        (recur)))

    (a/go-loop []
      (when-let [s (a/<! status-ch)]
        (printf "  status raw=%d%n" s)
        (recur)))

    (Thread/sleep 60000)
    (events/close-channel! c track-ch)
    (events/close-channel! c status-ch)
    (rb/disconnect c)))
