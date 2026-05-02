defmodule Rockbox.Error do
  @moduledoc """
  Base exception for the Rockbox SDK.

  Two specialised subclasses are raised in practice:

    * `Rockbox.NetworkError` — the HTTP request failed or returned a non-2xx
      status code (rockboxd unreachable, refused connection, timeout, …).
    * `Rockbox.GraphQLError` — rockboxd answered with a structured GraphQL
      error response. The original error list is available on the
      `:errors` field.

  Both inherit from `Rockbox.Error`, so a single rescue clause catches every
  SDK-originated failure:

      try do
        Rockbox.Playback.play!(client)
      rescue
        e in Rockbox.NetworkError -> Logger.error("offline: \#{e.message}")
        e in Rockbox.GraphQLError -> Logger.error("server error: \#{e.message}")
        e in Rockbox.Error        -> Logger.error("rockbox: \#{e.message}")
      end
  """

  defexception [:message, :cause]

  @type t :: %__MODULE__{message: String.t(), cause: term() | nil}
end

defmodule Rockbox.NetworkError do
  @moduledoc "Raised when the HTTP layer cannot reach rockboxd."

  defexception [:message, :cause]

  @type t :: %__MODULE__{message: String.t(), cause: term() | nil}
end

defmodule Rockbox.GraphQLError do
  @moduledoc "Raised when rockboxd returns a structured GraphQL error response."

  defexception [:message, :errors]

  @type error_detail :: %{
          required(:message) => String.t(),
          optional(:path) => [String.t() | non_neg_integer()],
          optional(:locations) => [%{line: non_neg_integer(), column: non_neg_integer()}],
          optional(:extensions) => map()
        }

  @type t :: %__MODULE__{message: String.t(), errors: [error_detail()]}

  @impl true
  def exception(errors) when is_list(errors) do
    %__MODULE__{
      message: errors |> Enum.map_join("; ", &Map.get(&1, :message, "GraphQL error")),
      errors: errors
    }
  end
end
