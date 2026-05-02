(ns ex10-smart-playlist
  "Create a 'recently played' smart playlist using a Clojure data rule set —
  the SDK encodes it to JSON for you."
  (:require [example-client :as client]
            [rockbox.smart-playlists :as sp]))

(defn -main [& _]
  (let [c (client/make-client)
        new-pl (sp/create c
                          {:name  "Recently played"
                           :rules {:operator "AND"
                                   :rules    [{:field "play_count"  :op "gt"     :value 0}
                                              {:field "last_played" :op "within" :value "30d"}]}})]
    (printf "Created %s (%s)%n" (:name new-pl) (:id new-pl))

    (printf "Currently resolves to %d tracks%n"
            (count (sp/track-ids c (:id new-pl))))

    ;; Cleanup so the example is repeatable
    (sp/delete c (:id new-pl))
    (println "Deleted.")))
