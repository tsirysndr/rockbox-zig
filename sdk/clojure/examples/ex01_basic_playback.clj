(ns ex01-basic-playback
  "Pause, seek to 1:30, resume — using the threading macro for a pipe-friendly
  call chain. Action functions return the client so they compose with `->`."
  (:require [example-client :as client]
            [rockbox.core     :as rb]
            [rockbox.playback :as pb]
            [rockbox.types    :as t]))

(defn -main [& _]
  (let [c (client/make-client)]
    (println "Status:" (pb/status c))
    (when-let [track (pb/current-track c)]
      (printf "Now playing: %s — %s (%s / %s)%n"
              (:title track) (:artist track)
              (t/format-ms (:elapsed track)) (t/format-ms (:length track))))

    (-> c
        (pb/pause)
        (pb/seek 90000)   ; jump to 1:30
        (pb/resume))

    (println "Status after pipe:" (pb/status c))))
