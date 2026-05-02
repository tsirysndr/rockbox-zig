import gleam/json
import gleam/string

/// Errors returned from any SDK call.
///
/// Pattern-match in your callers to react differently to network problems vs.
/// upstream GraphQL errors:
///
/// ```gleam
/// case playback.current_track(client) {
///   Ok(track) -> echo track
///   Error(error.NetworkError(_)) -> retry()
///   Error(error.GraphQLError(messages)) -> log_errors(messages)
///   Error(_) -> Nil
/// }
/// ```
pub type Error {
  /// Could not reach the server (DNS, refused connection, TLS, etc).
  NetworkError(reason: String)

  /// Server returned a non-2xx HTTP response.
  HttpError(status: Int, body: String)

  /// GraphQL endpoint returned a populated `errors` array.
  GraphQLError(messages: List(String))

  /// Response body could not be decoded into the expected shape.
  DecodeError(reason: String)
}

@internal
pub fn from_json_decode_error(err: json.DecodeError) -> Error {
  DecodeError(string.inspect(err))
}
