(ns rockbox.bluetooth
  "Bluetooth devices (Linux only — wraps BlueZ via D-Bus)."
  (:require [rockbox.transport :as t]))

(def ^:private device-fields
  "fragment BluetoothDeviceFields on BluetoothDevice {
     address name paired trusted connected rssi
   }")

(defn devices
  "List paired/known Bluetooth devices."
  [client]
  (:bluetooth-devices
   (t/execute client (str device-fields
                          " query BluetoothDevices { bluetoothDevices { ...BluetoothDeviceFields } }"))))

(defn scan
  "Scan for nearby Bluetooth devices. Optional `timeout-secs`."
  ([client] (scan client nil))
  ([client timeout-secs]
   (:bluetooth-scan
    (t/execute client
               (str device-fields
                    " mutation BluetoothScan($timeoutSecs: Int) {
                        bluetoothScan(timeoutSecs: $timeoutSecs) { ...BluetoothDeviceFields }
                      }")
               {:timeout-secs timeout-secs}))))

(defn connect
  "Connect to a Bluetooth device by address."
  [client address]
  (t/execute client
             "mutation BluetoothConnect($address: String!) { bluetoothConnect(address: $address) }"
             {:address address})
  client)

(defn disconnect
  "Disconnect a Bluetooth device by address."
  [client address]
  (t/execute client
             "mutation BluetoothDisconnect($address: String!) { bluetoothDisconnect(address: $address) }"
             {:address address})
  client)
