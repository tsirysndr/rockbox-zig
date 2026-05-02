defmodule Rockbox.Transport do
  @moduledoc """
  Low-level GraphQL HTTP transport. You normally don't call this directly —
  the `Rockbox.*` API modules wrap it. Exposed because `Rockbox.query/3` and
  the plugin system need a stable hook for raw queries.
  """

  alias Rockbox.{Client, Error, GraphQLError, NetworkError, Util}

  @type variables :: map() | Keyword.t() | nil

  @doc """
  Execute a GraphQL operation and return `{:ok, data}` or `{:error, exception}`.

  `data` is returned as-is (a map), not coerced to a struct — the API modules
  handle struct conversion. Variables are passed through `Rockbox.Util.camelize/1`
  to convert snake_case keys to the camelCase the server expects.
  """
  @spec execute(Client.t(), String.t(), variables()) ::
          {:ok, map()} | {:error, Exception.t()}
  def execute(%Client{} = client, query, variables \\ nil) do
    body = %{query: query, variables: encode_variables(variables)}

    case do_post(client, body) do
      {:ok, %{status: status, body: body}} when status in 200..299 ->
        case body do
          %{"errors" => [_ | _] = errors} ->
            {:error, GraphQLError.exception(Util.atomize(errors))}

          %{"data" => data} ->
            {:ok, data}

          other ->
            {:error, %Error{message: "Unexpected GraphQL response shape: #{inspect(other)}"}}
        end

      {:ok, %{status: status, body: body}} ->
        {:error,
         %NetworkError{
           message: "HTTP #{status} from #{client.http_url}",
           cause: body
         }}

      {:error, reason} ->
        {:error,
         %NetworkError{
           message: "Failed to reach rockboxd at #{client.http_url}",
           cause: reason
         }}
    end
  end

  @doc "Same as `execute/3` but raises on error."
  @spec execute!(Client.t(), String.t(), variables()) :: map()
  def execute!(client, query, variables \\ nil) do
    case execute(client, query, variables) do
      {:ok, data} -> data
      {:error, exception} -> raise exception
    end
  end

  # ---------------------------------------------------------------------------
  # Internal
  # ---------------------------------------------------------------------------

  defp do_post(%Client{} = client, body) do
    headers = [
      {"content-type", "application/json"},
      {"accept", "application/json"} | client.headers
    ]

    Req.post(
      client.http_url,
      json: body,
      headers: headers,
      receive_timeout: client.timeout,
      retry: false
    )
  end

  defp encode_variables(nil), do: nil
  defp encode_variables(vars) when map_size(vars) == 0, do: nil
  defp encode_variables(vars), do: Util.camelize(vars)
end
