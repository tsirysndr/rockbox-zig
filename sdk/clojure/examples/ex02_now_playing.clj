(ns ex02-now-playing
  "Fetch the currently playing track and pretty-print its details."
  (:require [example-client :as client]
            [rockbox.playback :as pb]
            [rockbox.types    :as t]))

(defn -main [& _]
  (let [c     (client/make-client)
        track (pb/current-track c)]
    (if track
      (do
        (println "──────── Now playing ────────")
        (printf "  Title    : %s%n" (:title track))
        (printf "  Artist   : %s%n" (:artist track))
        (printf "  Album    : %s (%d)%n" (:album track) (or (:year track) 0))
        (printf "  Position : %s / %s%n"
                (t/format-ms (:elapsed track))
                (t/format-ms (:length track)))
        (printf "  Bitrate  : %d kbps @ %d Hz%n"
                (or (:bitrate track) 0) (or (:frequency track) 0))
        (printf "  Path     : %s%n" (:path track)))
      (println "Nothing is playing."))))
