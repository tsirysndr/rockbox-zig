(ns ex04-queue-management
  "Inspect and modify the live playback queue."
  (:require [example-client :as client]
            [rockbox.playlist :as q]
            [rockbox.types    :as t]))

(defn -main [& _]
  (let [c (client/make-client)
        {:keys [tracks index amount]} (q/current c)]
    (printf "Queue — %d tracks, currently at index %d%n%n" amount index)
    (doseq [[i tr] (map-indexed vector (take 10 tracks))]
      (printf "%s %2d. %s — %s  [%s]%n"
              (if (= i index) "▶" " ")
              (inc i) (:title tr) (:artist tr)
              (t/format-ms (:length tr))))

    ;; Pipe-friendly mutation chain
    (-> c
        (q/insert-tracks ["/Music/example.mp3"] :next)
        (q/shuffle))

    (println "\nQueue after shuffle:" (q/amount c))))
