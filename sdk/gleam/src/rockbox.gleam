//// Rockbox Gleam SDK — pipe-friendly client for the rockboxd GraphQL API.
////
//// ```gleam
//// import rockbox
//// import rockbox/playback
////
//// pub fn main() {
////   let client = rockbox.connect()
////
////   let assert Ok(track) = playback.current_track(client)
////   let assert Ok(_) = playback.pause(client)
//// }
//// ```
////
//// Customise the connection with the builder:
////
//// ```gleam
//// let client =
////   rockbox.new()
////   |> rockbox.host("rockbox.local")
////   |> rockbox.port(8080)
////   |> rockbox.connect
//// ```

import gleam/dynamic/decode
import gleam/int
import gleam/json.{type Json}
import gleam/option.{type Option, None, Some}
import rockbox/error.{type Error}
import rockbox/internal/transport.{type Transport, Transport}

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

/// A configured client. Hand it to any of the per-domain API modules:
///
/// ```gleam
/// playback.play(client, 0, 0)
/// library.search(client, "miles davis")
/// ```
pub opaque type Client {
  Client(transport: Transport)
}

/// A fluent builder used to configure a `Client`.
///
/// Construct with `new`, override defaults with `host` / `port` / `url`, then
/// call `connect` (or `build`, the alias) to get a `Client`.
pub opaque type Builder {
  Builder(host: String, port: Int, url_override: Option(String))
}

// ---------------------------------------------------------------------------
// Builder API
// ---------------------------------------------------------------------------

/// Start a new client builder with sensible defaults (`localhost:6062`).
pub fn new() -> Builder {
  Builder(host: "localhost", port: 6062, url_override: None)
}

/// Override the hostname (default `"localhost"`). Ignored if `url` is set.
pub fn host(builder: Builder, value: String) -> Builder {
  Builder(..builder, host: value)
}

/// Override the port (default `6062`). Ignored if `url` is set.
pub fn port(builder: Builder, value: Int) -> Builder {
  Builder(..builder, port: value)
}

/// Override the full GraphQL HTTP URL. Takes precedence over `host` / `port`.
pub fn url(builder: Builder, value: String) -> Builder {
  Builder(..builder, url_override: Some(value))
}

/// Finalise the builder and return a usable `Client`.
pub fn connect(builder: Builder) -> Client {
  let http_url = case builder.url_override {
    Some(value) -> value
    None ->
      "http://"
      <> builder.host
      <> ":"
      <> int.to_string(builder.port)
      <> "/graphql"
  }
  Client(transport: Transport(http_url: http_url))
}

/// Alias for `connect`. Use whichever name reads better in your code.
pub fn build(builder: Builder) -> Client {
  connect(builder)
}

// ---------------------------------------------------------------------------
// Convenience constructors
// ---------------------------------------------------------------------------

/// Shortcut for `new() |> connect()` — a client pointed at `localhost:6062`.
pub fn default_client() -> Client {
  new() |> connect
}

/// Shortcut for `new() |> host(host) |> port(port) |> connect()`.
pub fn at(host h: String, port p: Int) -> Client {
  new() |> host(h) |> port(p) |> connect
}

// ---------------------------------------------------------------------------
// Escape hatches & accessors
// ---------------------------------------------------------------------------

/// Run a raw GraphQL operation against the server, decoding the `data` field
/// with the supplied decoder. Useful if you need an endpoint the SDK doesn't
/// expose directly yet.
///
/// ```gleam
/// import gleam/dynamic/decode
/// import gleam/json
///
/// let decoder = {
///   use version <- decode.field("rockboxVersion", decode.string)
///   decode.success(version)
/// }
///
/// let assert Ok(version) =
///   client
///   |> rockbox.query("query { rockboxVersion }", json.object([]), decoder)
/// ```
pub fn query(
  client: Client,
  gql: String,
  variables: Json,
  decoder: decode.Decoder(t),
) -> Result(t, Error) {
  transport.execute(client.transport, gql, variables, decoder)
}

/// Like `query` but discards the response body — handy for fire-and-forget
/// mutations that return a boolean status flag you don't care about.
pub fn execute(
  client: Client,
  gql: String,
  variables: Json,
) -> Result(Nil, Error) {
  transport.execute_unit(client.transport, gql, variables)
}

/// Return the underlying GraphQL HTTP URL. Useful for diagnostics & tests.
pub fn http_url(client: Client) -> String {
  client.transport.http_url
}

/// Internal: hand the underlying transport to other modules in the SDK.
@internal
pub fn transport(client: Client) -> Transport {
  client.transport
}

