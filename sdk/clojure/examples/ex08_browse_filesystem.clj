(ns ex08-browse-filesystem
  "Walk the configured music_dir and print directories vs files."
  (:require [example-client :as client]
            [rockbox.browse :as br]))

(defn -main [& args]
  (let [c    (client/make-client)
        path (or (first args) nil)]
    (printf "── %s ──%n" (or path "<music_dir root>"))
    (doseq [d (br/directories c path)]
      (printf "📁 %s%n" (:name d)))
    (doseq [f (br/files c path)]
      (printf "🎵 %s%n" (:name f)))))
