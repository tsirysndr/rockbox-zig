(ns rockbox.errors
  "Typed exceptions for the Rockbox SDK.

  All errors are `clojure.lang.ExceptionInfo` instances carrying a `:type`
  key in their ex-data, so they can be discriminated with `ex-data` /
  `(:type ...)` in a single `catch ExceptionInfo` block.

      (try
        (rb/query client \"...\")
        (catch clojure.lang.ExceptionInfo e
          (case (:type (ex-data e))
            :rockbox/network (handle-offline e)
            :rockbox/graphql (handle-server-error e)
            (throw e))))")

(defn network-error
  "Throwable: rockboxd is unreachable, or HTTP returned a non-2xx status."
  ([msg]       (network-error msg nil))
  ([msg cause]
   (ex-info msg
            {:type  :rockbox/network
             :cause cause}
            (when (instance? Throwable cause) cause))))

(defn graphql-error
  "Throwable: server returned `{errors: [...]}` in the response body.
  `errors` is the raw vector from the server (kebabized maps)."
  [errors]
  (ex-info (->> errors (map :message) (clojure.string/join "; "))
           {:type   :rockbox/graphql
            :errors errors}))

(defn config-error
  "Throwable: client was constructed with bad/missing config."
  [msg]
  (ex-info msg {:type :rockbox/config}))

(defn network-error?
  "True if `e` is a Rockbox network error."
  [e]
  (and (instance? clojure.lang.ExceptionInfo e)
       (= :rockbox/network (:type (ex-data e)))))

(defn graphql-error?
  "True if `e` is a Rockbox GraphQL error."
  [e]
  (and (instance? clojure.lang.ExceptionInfo e)
       (= :rockbox/graphql (:type (ex-data e)))))
