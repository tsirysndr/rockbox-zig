(ns build
  (:require [clojure.tools.build.api :as b]
            [deps-deploy.deps-deploy :as dd]))

(def lib 'org.clojars.tsiry/rockbox-clj)
(def version (or (System/getenv "VERSION") "0.1.0"))
(def class-dir "target/classes")
(def basis (delay (b/create-basis {:project "deps.edn"})))
(def jar-file (format "target/%s-%s.jar" (name lib) version))

(defn clean [_]
  (b/delete {:path "target"}))

(defn jar [_]
  (b/write-pom {:class-dir class-dir
                :lib       lib
                :version   version
                :basis     @basis
                :src-dirs  ["src"]
                :scm       {:url                 "https://github.com/tsirysndr/rockboxd"
                            :connection          "scm:git:git://github.com/tsirysndr/rockboxd.git"
                            :developerConnection "scm:git:ssh://git@github.com/tsirysndr/rockboxd.git"
                            :tag                 (str "clojure-v" version)}
                :pom-data  [[:description "Idiomatic Clojure SDK for Rockbox — GraphQL client with WebSocket subscriptions and a tiny plugin system."]
                            [:url "https://github.com/tsirysndr/rockboxd"]
                            [:licenses
                             [:license
                              [:name "MIT License"]
                              [:url "https://opensource.org/licenses/MIT"]]]]})
  (b/copy-dir {:src-dirs   ["src"]
               :target-dir class-dir})
  (b/jar {:class-dir class-dir
          :jar-file  jar-file}))

(defn install [_]
  (clean nil)
  (jar nil)
  (b/install {:basis     @basis
              :lib       lib
              :version   version
              :jar-file  jar-file
              :class-dir class-dir}))

(defn deploy [_]
  (clean nil)
  (jar nil)
  (dd/deploy {:installer :remote
              :artifact  (b/resolve-path jar-file)
              :pom-file  (b/pom-path {:lib lib :class-dir class-dir})}))
