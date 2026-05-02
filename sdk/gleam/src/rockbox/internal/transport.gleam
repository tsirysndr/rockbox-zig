//// Internal HTTP transport for the Rockbox GraphQL endpoint.
////
//// Not part of the public API — call sites should use the per-domain
//// modules under `rockbox/*`.

import gleam/dynamic
import gleam/dynamic/decode
import gleam/http.{Post}
import gleam/http/request
import gleam/httpc
import gleam/json.{type Json}
import gleam/list
import gleam/option.{type Option, None, Some}
import gleam/result
import gleam/string
import rockbox/error.{type Error}

pub type Transport {
  Transport(http_url: String)
}

/// Build the JSON variables payload from a list of optional pairs.
///
/// Pairs whose value is `None` are dropped, so the GraphQL endpoint never sees
/// e.g. `"description": null` for an unset optional argument.
pub fn variables(pairs: List(#(String, Option(Json)))) -> Json {
  pairs
  |> list.filter_map(fn(pair) {
    let #(key, value) = pair
    case value {
      Some(json) -> Ok(#(key, json))
      None -> Error(Nil)
    }
  })
  |> json.object
}

/// Execute a GraphQL operation. The response is decoded with `decoder` against
/// the contents of the top-level `data` field.
pub fn execute(
  transport: Transport,
  query: String,
  vars: Json,
  decoder: decode.Decoder(t),
) -> Result(t, Error) {
  let body =
    json.object([#("query", json.string(query)), #("variables", vars)])
    |> json.to_string

  use req <- result.try(
    request.to(transport.http_url)
    |> result.replace_error(error.NetworkError(
      "invalid URL: " <> transport.http_url,
    )),
  )

  let req =
    req
    |> request.set_method(Post)
    |> request.set_header("content-type", "application/json")
    |> request.set_header("accept", "application/json")
    |> request.set_body(body)

  use response <- result.try(
    httpc.send(req)
    |> result.map_error(fn(err) { error.NetworkError(string.inspect(err)) }),
  )

  case response.status {
    s if s >= 200 && s < 300 -> decode_response(response.body, decoder)
    s -> Error(error.HttpError(s, response.body))
  }
}

/// A query returning no useful data — discards the decoded value.
pub fn execute_unit(
  transport: Transport,
  query: String,
  vars: Json,
) -> Result(Nil, Error) {
  execute(transport, query, vars, decode.dynamic)
  |> result.replace(Nil)
}

fn decode_response(
  body: String,
  decoder: decode.Decoder(t),
) -> Result(t, Error) {
  let envelope_decoder = {
    use errors <- decode.optional_field(
      "errors",
      [],
      decode.list(decode.at(["message"], decode.string)),
    )
    use data <- decode.optional_field("data", dynamic.nil(), decode.dynamic)
    decode.success(#(errors, data))
  }

  case json.parse(body, envelope_decoder) {
    Ok(#([], data)) ->
      decode.run(data, decoder)
      |> result.map_error(fn(errs) { error.DecodeError(string.inspect(errs)) })
    Ok(#(messages, _)) -> Error(error.GraphQLError(messages))
    Error(err) -> Error(error.from_json_decode_error(err))
  }
}
