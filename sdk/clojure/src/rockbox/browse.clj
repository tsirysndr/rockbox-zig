(ns rockbox.browse
  "Filesystem browser — walk the configured `music_dir`."
  (:require [rockbox.transport :as t]
            [rockbox.types     :as types]))

(defn entries
  "All entries (files + directories) under `path` (or root of music_dir)."
  ([client]      (entries client nil))
  ([client path]
   (:tree-get-entries
    (t/execute client
               "query Browse($path: String) {
                  treeGetEntries(path: $path) { name attr timeWrite customaction displayName }
                }"
               {:path path}))))

(defn directories
  "Only directory entries."
  ([client]      (directories client nil))
  ([client path] (filterv types/directory? (entries client path))))

(defn files
  "Only file entries."
  ([client]      (files client nil))
  ([client path] (filterv types/file? (entries client path))))
