(ns rockbox.system
  "System info — daemon version and global runtime status."
  (:require [rockbox.transport :as t]))

(defn version
  "rockboxd version string."
  [client]
  (:rockbox-version (t/execute client "query Version { rockboxVersion }")))

(defn status
  "Global runtime status: `:runtime :topruntime :resume-index ...`."
  [client]
  (:global-status
   (t/execute client
              "query GlobalStatus {
                 globalStatus {
                   resumeIndex resumeCrc32 resumeElapsed resumeOffset
                   runtime topruntime dircacheSize
                   lastScreen viewerIconCount lastVolumeChange
                 }
               }")))
