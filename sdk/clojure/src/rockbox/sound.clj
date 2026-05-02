(ns rockbox.sound
  "Volume control. Values are in firmware-defined steps, not absolute dB."
  (:require [rockbox.transport :as t]))

(defn volume
  "Current volume info: `{:volume :min :max}`."
  [client]
  (:volume (t/execute client "query Volume { volume { volume min max } }")))

(defn adjust-volume
  "Change volume by `steps` (positive = louder, negative = quieter).
  Returns the new raw volume value."
  [client steps]
  (:adjust-volume
   (t/execute client "mutation AdjustVolume($steps: Int!) { adjustVolume(steps: $steps) }"
              {:steps steps})))

(defn volume-up
  "Step volume up by one. Returns the new raw volume."
  [client] (adjust-volume client  1))

(defn volume-down
  "Step volume down by one. Returns the new raw volume."
  [client] (adjust-volume client -1))
