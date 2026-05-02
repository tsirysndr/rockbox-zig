defmodule Rockbox.Client do
  @moduledoc """
  The Rockbox client struct — a stateless handle that holds the URLs and an
  optional Req client. Construct with `Rockbox.new/0` or `Rockbox.new/1`.

  All API functions in `Rockbox.*` accept this as their first argument so that
  calls compose cleanly with the pipe operator.

      iex> client = Rockbox.new()
      iex> client.http_url
      "http://localhost:6062/graphql"
  """

  @typedoc """
  Configuration options accepted by `Rockbox.new/1`.

    * `:host`     — hostname or IP of rockboxd. Default: `"localhost"`.
    * `:port`     — GraphQL HTTP/WS port. Default: `6062`.
    * `:http_url` — full HTTP URL override (takes precedence over host/port).
    * `:ws_url`   — full WebSocket URL override (takes precedence over host/port).
    * `:headers`  — extra HTTP headers (list of `{key, value}` tuples).
    * `:timeout`  — request timeout in ms. Default: `15_000`.
  """
  @type opts :: [
          host: String.t(),
          port: non_neg_integer(),
          http_url: String.t(),
          ws_url: String.t(),
          headers: [{String.t(), String.t()}],
          timeout: non_neg_integer()
        ]

  @type t :: %__MODULE__{
          host: String.t(),
          port: non_neg_integer(),
          http_url: String.t(),
          ws_url: String.t(),
          headers: [{String.t(), String.t()}],
          timeout: non_neg_integer()
        }

  defstruct [
    :host,
    :port,
    :http_url,
    :ws_url,
    headers: [],
    timeout: 15_000
  ]

  @doc "Build a client from options. See `t:opts/0`."
  @spec new(opts()) :: t()
  def new(opts \\ []) do
    host = Keyword.get(opts, :host, "localhost")
    port = Keyword.get(opts, :port, 6062)
    http_url = Keyword.get(opts, :http_url, "http://#{host}:#{port}/graphql")
    ws_url = Keyword.get(opts, :ws_url, "ws://#{host}:#{port}/graphql")

    %__MODULE__{
      host: host,
      port: port,
      http_url: http_url,
      ws_url: ws_url,
      headers: Keyword.get(opts, :headers, []),
      timeout: Keyword.get(opts, :timeout, 15_000)
    }
  end
end
