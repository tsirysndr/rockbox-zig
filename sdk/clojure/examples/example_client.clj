(ns example-client
  "Shared client used by the example scripts.

  Run an example with:

      clj -M:examples -m ex01-basic-playback

  Override the host/port via env vars:

      ROCKBOX_HOST=192.168.1.42 ROCKBOX_PORT=6062 clj -M:examples -m ex01-basic-playback"
  (:require [rockbox.core :as rb]))

(defn make-client []
  (rb/client {:host (or (System/getenv "ROCKBOX_HOST") "localhost")
              :port (Integer/parseInt (or (System/getenv "ROCKBOX_PORT") "6062"))}))
