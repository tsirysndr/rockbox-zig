(ns ex09-plugin-scrobbler
  "A toy scrobbler plugin that reports plays > 30 s old."
  (:require [example-client :as client]
            [rockbox.core :as rb]))

;; A plugin is a plain map. Compose new ones with `assoc` / `merge`.
(defn scrobbler [submit-fn]
  (let [state (atom {:current nil :started-at 0})]
    {:name        "scrobbler"
     :version     "0.1.0"
     :description "Submits plays > 30 s old to `submit-fn`"
     :install     (fn [{:keys [events]}]
                    ((:on events) :track-changed
                     (fn [tr]
                       (let [{:keys [current started-at]} @state]
                         (when (and current
                                    (> (- (System/currentTimeMillis) started-at) 30000))
                           (submit-fn current))
                         (reset! state {:current tr :started-at (System/currentTimeMillis)})))))
     :uninstall   (fn [] (reset! state {:current nil :started-at 0}))}))

(defn -main [& _]
  (let [c (client/make-client)]
    (-> c
        (rb/connect)
        (rb/use-plugin (scrobbler (fn [tr]
                                    (printf "📡 scrobble %s — %s%n"
                                            (:title tr) (:artist tr))))))
    (println "Installed plugins:" (mapv :name (rb/installed-plugins c)))
    (println "Listening for 5 minutes…")
    (Thread/sleep (* 5 60 1000))
    (rb/unuse-plugin c "scrobbler")
    (rb/disconnect c)))
