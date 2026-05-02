//// Bluetooth device pairing and connection (Linux only).

import gleam/dynamic/decode
import gleam/json
import gleam/option.{type Option}
import rockbox.{type Client}
import rockbox/error.{type Error}
import rockbox/internal/transport
import rockbox/types.{type BluetoothDevice}

const bluetooth_fields = "
  fragment BluetoothDeviceFields on BluetoothDevice {
    address name paired trusted connected rssi
  }
"

/// Known/paired devices.
pub fn devices(client: Client) -> Result(List(BluetoothDevice), Error) {
  let decoder = {
    use devices <- decode.field(
      "bluetoothDevices",
      decode.list(types.bluetooth_device_decoder()),
    )
    decode.success(devices)
  }
  let q = bluetooth_fields <> "
    query BluetoothDevices { bluetoothDevices { ...BluetoothDeviceFields } }
  "
  rockbox.query(client, q, json.object([]), decoder)
}

/// Trigger an active scan. `timeout_secs` of `option.None` uses the firmware
/// default.
pub fn scan(
  client: Client,
  timeout_secs: Option(Int),
) -> Result(List(BluetoothDevice), Error) {
  let decoder = {
    use devices <- decode.field(
      "bluetoothScan",
      decode.list(types.bluetooth_device_decoder()),
    )
    decode.success(devices)
  }
  let vars =
    transport.variables([
      #("timeoutSecs", option.map(timeout_secs, json.int)),
    ])
  let q = bluetooth_fields <> "
    mutation BluetoothScan($timeoutSecs: Int) {
      bluetoothScan(timeoutSecs: $timeoutSecs) { ...BluetoothDeviceFields }
    }
  "
  rockbox.query(client, q, vars, decoder)
}

pub fn connect(client: Client, address: String) -> Result(Nil, Error) {
  rockbox.execute(
    client,
    "mutation BluetoothConnect($address: String!) { bluetoothConnect(address: $address) }",
    json.object([#("address", json.string(address))]),
  )
}

pub fn disconnect(client: Client, address: String) -> Result(Nil, Error) {
  rockbox.execute(
    client,
    "mutation BluetoothDisconnect($address: String!) { bluetoothDisconnect(address: $address) }",
    json.object([#("address", json.string(address))]),
  )
}
