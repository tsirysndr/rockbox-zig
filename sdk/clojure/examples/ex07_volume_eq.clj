(ns ex07-volume-eq
  "Volume control + a tasteful EQ preset, all in one piped chain."
  (:require [example-client :as client]
            [rockbox.sound    :as snd]
            [rockbox.settings :as settings]))

(defn -main [& _]
  (let [c (client/make-client)
        {:keys [volume min max]} (snd/volume c)]
    (printf "Current volume: %d (range %d..%d)%n" volume min max)
    (printf "Bumped to:      %d%n" (snd/volume-up c))

    (settings/save c
                   {:eq-enabled       true
                    :eq-precut        -3
                    :eq-band-settings [{:cutoff   60 :q 7 :gain  3}   ; bass boost
                                       {:cutoff  200 :q 7 :gain  0}
                                       {:cutoff  800 :q 7 :gain  0}
                                       {:cutoff 4000 :q 7 :gain -2}   ; tame the presence band
                                       {:cutoff 12000 :q 7 :gain  1}]})

    (println "EQ enabled with a 5-band preset.")))
