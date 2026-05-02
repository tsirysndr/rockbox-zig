(ns rockbox.util
  "Internal helpers — case conversion between Clojure (kebab-case keywords)
  and GraphQL (camelCase strings)."
  (:require [clojure.string :as str]))

;; ---------------------------------------------------------------------------
;; Case conversion
;; ---------------------------------------------------------------------------

(defn kebab->camel
  "kebab-case-name -> kebabCaseName. Pass-through for strings without dashes."
  ^String [^String s]
  (let [parts (str/split s #"-")]
    (if (= 1 (count parts))
      s
      (apply str (first parts) (map str/capitalize (rest parts))))))

(defn camel->kebab
  "camelCaseName -> kebab-case-name."
  ^String [^String s]
  (-> s
      (str/replace #"([a-z0-9])([A-Z])" "$1-$2")
      (str/lower-case)))

(defn- key->camel-string [k]
  (cond
    (keyword? k) (kebab->camel (name k))
    (string?  k) (kebab->camel k)
    :else        k))

(defn- key->kebab-keyword [k]
  (cond
    (keyword? k) (keyword (camel->kebab (name k)))
    (string?  k) (keyword (camel->kebab k))
    :else        k))

(defn camelize-keys
  "Recursively convert all map keys to camelCase strings (for GraphQL variables)."
  [x]
  (cond
    (map? x)        (into {} (map (fn [[k v]] [(key->camel-string k) (camelize-keys v)]) x))
    (sequential? x) (mapv camelize-keys x)
    :else           x))

(defn kebabize-keys
  "Recursively convert all map keys to kebab-case keywords (for response data)."
  [x]
  (cond
    (map? x)        (into {} (map (fn [[k v]] [(key->kebab-keyword k) (kebabize-keys v)]) x))
    (sequential? x) (mapv kebabize-keys x)
    :else           x))

;; ---------------------------------------------------------------------------
;; GraphQL helpers
;; ---------------------------------------------------------------------------

(defn drop-nils
  "Strip nil values from a map. Useful when forwarding optional GraphQL args."
  [m]
  (into {} (remove (comp nil? val) m)))
