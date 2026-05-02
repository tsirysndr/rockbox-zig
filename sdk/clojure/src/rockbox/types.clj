(ns rockbox.types
  "Enum constants and tiny helpers for Rockbox values that come back as ints
  or attribute bitmasks.

  Constants are exposed both as namespaced keywords (idiomatic in Clojure)
  and as the raw integers the firmware uses. Use whichever fits your code:

      (require '[rockbox.types :as t])

      (= (:status track) (t/playback-status :playing))     ; via keyword
      (= (:status track) t/playing)                         ; via raw int alias
")

;; ---------------------------------------------------------------------------
;; Playback status (firmware enum)
;; ---------------------------------------------------------------------------

(def playback-status
  "Map :stopped/:playing/:paused -> firmware integer."
  {:stopped 0
   :playing 1
   :paused  3})

(def playback-status->keyword
  "Reverse lookup — firmware integer -> keyword."
  (zipmap (vals playback-status) (keys playback-status)))

(def stopped 0)
(def playing 1)
(def paused  3)

;; ---------------------------------------------------------------------------
;; Repeat mode
;; ---------------------------------------------------------------------------

(def repeat-mode
  {:off       0
   :all       1
   :one       2
   :shuffle   3
   :ab-repeat 4})

(def repeat-mode->keyword
  (zipmap (vals repeat-mode) (keys repeat-mode)))

;; ---------------------------------------------------------------------------
;; Channel config
;; ---------------------------------------------------------------------------

(def channel-config
  {:stereo        0
   :stereo-narrow 1
   :mono          2
   :left-mix      3
   :right-mix     4
   :karaoke       5})

;; ---------------------------------------------------------------------------
;; ReplayGain type
;; ---------------------------------------------------------------------------

(def replaygain-type
  {:track   0
   :album   1
   :shuffle 2})

;; ---------------------------------------------------------------------------
;; Insert position (queue management)
;; ---------------------------------------------------------------------------

(def insert-position
  "Where to insert tracks into the active queue.

    :next          — after the currently playing track
    :after-current — after the last manually inserted track
    :last          — at the end of the queue
    :first         — replace the entire queue"
  {:next          0
   :after-current 1
   :last          2
   :first         3})

(defn ->insert-position
  "Coerce a keyword or int into the firmware integer."
  [x]
  (if (integer? x) x (get insert-position x 0)))

;; ---------------------------------------------------------------------------
;; Filesystem entry helpers
;; ---------------------------------------------------------------------------

(def ^:const directory-attr-bit 0x10)

(defn directory?
  "True if the entry is a directory (bit 4 of `:attr`)."
  [entry]
  (not (zero? (bit-and (or (:attr entry) 0) directory-attr-bit))))

(defn file?
  "True if the entry is a file (i.e. not a directory)."
  [entry]
  (not (directory? entry)))

;; ---------------------------------------------------------------------------
;; Duration formatting
;; ---------------------------------------------------------------------------

(defn format-ms
  "Format a millisecond duration as `M:SS`.

      (format-ms 75000) ;=> \"1:15\""
  [ms]
  (if (and (number? ms) (not (neg? ms)))
    (let [total   (long (quot ms 1000))
          minutes (quot total 60)
          seconds (rem  total 60)]
      (format "%d:%02d" minutes seconds))
    "0:00"))
