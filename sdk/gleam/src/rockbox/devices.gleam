//// Discovery and routing for Chromecast / AirPlay / UPnP / Snapcast endpoints.

import gleam/dynamic/decode
import gleam/json
import gleam/option.{type Option}
import rockbox.{type Client}
import rockbox/error.{type Error}
import rockbox/types.{type Device}

const device_fields = "
  id name host ip port service app isConnected
  baseUrl isCastDevice isSourceDevice isCurrentDevice
"

pub fn list(client: Client) -> Result(List(Device), Error) {
  let decoder = {
    use devices <- decode.field("devices", decode.list(types.device_decoder()))
    decode.success(devices)
  }
  rockbox.query(
    client,
    "query Devices { devices { " <> device_fields <> " } }",
    json.object([]),
    decoder,
  )
}

pub fn get(client: Client, id: String) -> Result(Option(Device), Error) {
  let decoder = {
    use device <- decode.field(
      "device",
      decode.optional(types.device_decoder()),
    )
    decode.success(device)
  }
  rockbox.query(
    client,
    "query Device($id: String!) { device(id: $id) { " <> device_fields <> " } }",
    json.object([#("id", json.string(id))]),
    decoder,
  )
}

pub fn connect(client: Client, id: String) -> Result(Nil, Error) {
  rockbox.execute(
    client,
    "mutation ConnectDevice($id: String!) { connect(id: $id) }",
    json.object([#("id", json.string(id))]),
  )
}

pub fn disconnect(client: Client, id: String) -> Result(Nil, Error) {
  rockbox.execute(
    client,
    "mutation DisconnectDevice($id: String!) { disconnect(id: $id) }",
    json.object([#("id", json.string(id))]),
  )
}
