(ns ex03-library-search
  "Search the library and play the first matching album, shuffled."
  (:require [example-client :as client]
            [rockbox.library  :as lib]
            [rockbox.playback :as pb]))

(defn -main [& args]
  (let [term (or (first args) "radiohead")
        c    (client/make-client)
        {:keys [artists albums tracks]} (lib/search c term)]

    (printf "Searching for %s%n%n" (pr-str term))
    (printf "Artists (%d):%n" (count artists))
    (doseq [a (take 5 artists)] (printf "  • %s%n" (:name a)))

    (printf "%nAlbums (%d):%n" (count albums))
    (doseq [a (take 5 albums)] (printf "  • %s — %s (%d)%n" (:title a) (:artist a) (or (:year a) 0)))

    (printf "%nTracks (%d):%n" (count tracks))
    (doseq [t (take 5 tracks)] (printf "  • %s — %s%n" (:title t) (:artist t)))

    (when-let [first-album (first albums)]
      (printf "%n▶ Playing %s (shuffled)…%n" (:title first-album))
      (pb/play-album c (:id first-album) {:shuffle true}))))
