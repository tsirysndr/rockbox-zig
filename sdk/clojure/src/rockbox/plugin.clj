(ns rockbox.plugin
  "Plugin system inspired by Jellyfin's IPlugin and Mopidy's frontend
  extensions. A plugin is a plain map with at minimum a `:name` and
  `:install` function:

      (def my-scrobbler
        {:name        \"lastfm-scrobbler\"
         :version     \"1.0.0\"
         :description \"Scrobble plays to Last.fm\"
         :install     (fn [{:keys [client query events]}]
                        (events/on client :track-changed
                          (fn [t] (submit-scrobble t))))
         :uninstall   (fn [] (disconnect-lastfm))})

  Install with `rockbox.core/use-plugin`. The `:install` fn receives a
  context map: `{:client client :query query-fn :events events-ns}`."
  (:require [rockbox.transport :as transport]
            [rockbox.events    :as events]))

(defn- ensure-registry [client]
  (or (:plugins client)
      (throw (ex-info "Client missing :plugins atom" {:type :rockbox/config}))))

(defn install
  "Install `plugin` into `client`. Throws if a plugin with the same `:name`
  is already installed."
  [client plugin]
  (when-not (and (map? plugin) (string? (:name plugin)))
    (throw (ex-info "Plugin must be a map with a :name string"
                    {:type :rockbox/config :plugin plugin})))
  (let [registry (ensure-registry client)]
    (when (contains? @registry (:name plugin))
      (throw (ex-info (str "Plugin \"" (:name plugin) "\" is already installed")
                      {:type :rockbox/config :name (:name plugin)})))
    (let [ctx {:client client
               :query  (fn query
                         ([q]      (transport/execute client q))
                         ([q vars] (transport/execute client q vars)))
               :events {:on            (partial events/on            client)
                        :once          (partial events/once          client)
                        :off           (partial events/off           client)
                        :off-all       (partial events/off-all       client)
                        :channel       (partial events/channel       client)
                        :close-channel (partial events/close-channel! client)}}
          install-fn (:install plugin)]
      (when install-fn (install-fn ctx))
      (swap! registry assoc (:name plugin) plugin)
      client)))

(defn uninstall
  "Uninstall a plugin by `:name`. Returns the client. Idempotent."
  [client plugin-name]
  (let [registry (ensure-registry client)
        {:keys [uninstall] :as plugin} (get @registry plugin-name)]
    (when plugin
      (when uninstall (uninstall))
      (swap! registry dissoc plugin-name))
    client))

(defn installed
  "List installed plugins for `client`."
  [client]
  (vec (vals @(ensure-registry client))))
