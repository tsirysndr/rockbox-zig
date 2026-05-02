(ns rockbox.devices
  "Remote output sinks discovered via mDNS — Chromecast, AirPlay, etc."
  (:refer-clojure :exclude [list get])
  (:require [rockbox.transport :as t]))

(def ^:private device-fields
  "id name host ip port service app isConnected
   baseUrl isCastDevice isSourceDevice isCurrentDevice")

(defn list
  "All discovered devices."
  [client]
  (:devices
   (t/execute client (str "query Devices { devices { " device-fields " } }"))))

(defn get
  "Single device by id, or `nil`."
  [client id]
  (:device
   (t/execute client (str "query Device($id: String!) { device(id: $id) { " device-fields " } }")
              {:id id})))

(defn connect
  "Switch the active PCM output sink to this device."
  [client id]
  (t/execute client "mutation ConnectDevice($id: String!) { connect(id: $id) }" {:id id})
  client)

(defn disconnect
  "Revert to the built-in PCM sink."
  [client id]
  (t/execute client "mutation DisconnectDevice($id: String!) { disconnect(id: $id) }" {:id id})
  client)
