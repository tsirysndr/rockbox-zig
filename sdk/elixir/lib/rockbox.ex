defmodule Rockbox do
  @moduledoc """
  Idiomatic Elixir SDK for [Rockbox](https://www.rockbox.org).

  ## Quick start

      client = Rockbox.new()

      # Optional: open the WebSocket for real-time events
      {:ok, _} = Rockbox.connect(client)
      :ok = Rockbox.Events.subscribe(:track_changed)

      # Look at what's playing
      {:ok, track} = Rockbox.Playback.current_track(client)
      IO.inspect(track)

      # Search the library
      {:ok, results} = Rockbox.Library.search(client, "radiohead")

      # Play an album, shuffled
      :ok = Rockbox.Playback.play_album(client, hd(results.albums).id, shuffle: true)

      # React to track changes
      receive do
        {:rockbox, :track_changed, track} ->
          IO.puts("▶ \#{track.title} — \#{track.artist}")
      end

  ## Module map

  | Domain                | Module                     |
  |-----------------------|----------------------------|
  | Transport controls    | `Rockbox.Playback`         |
  | Library / search      | `Rockbox.Library`          |
  | Live queue            | `Rockbox.Queue`            |
  | Saved playlists       | `Rockbox.SavedPlaylists`   |
  | Smart playlists       | `Rockbox.SmartPlaylists`   |
  | Smart-playlist rules  | `Rockbox.SmartPlaylist.Rules` |
  | Volume                | `Rockbox.Sound`            |
  | Settings              | `Rockbox.Settings`         |
  | System info           | `Rockbox.System`           |
  | Filesystem browser    | `Rockbox.Browse`           |
  | Output devices        | `Rockbox.Devices`          |
  | Bluetooth (Linux)     | `Rockbox.Bluetooth`        |
  | Real-time events      | `Rockbox.Events`           |
  | Plugin behaviour      | `Rockbox.Plugin`           |

  Every API function takes the client as its first argument so it composes
  with the pipe operator. Functions that may fail return
  `{:ok, value} | {:error, exception}` and have a matching `name!/N` variant
  that raises on error.
  """

  alias Rockbox.{Client, Plugins, Socket, Transport}

  # ---------------------------------------------------------------------------
  # Client construction
  # ---------------------------------------------------------------------------

  @doc """
  Build a new client. See `Rockbox.Client` for the full option list.

      iex> client = Rockbox.new()
      iex> client.host
      "localhost"
  """
  @spec new(Client.opts()) :: Client.t()
  def new(opts \\ []), do: Client.new(opts)

  # ---------------------------------------------------------------------------
  # Real-time subscriptions
  # ---------------------------------------------------------------------------

  @doc """
  Open the WebSocket connection so subscribers start receiving events.

  Idempotent — calling it twice for the same client returns the existing pid.
  Subscribe with `Rockbox.Events.subscribe/1` (or `Rockbox.subscribe/1`).
  """
  @spec connect(Client.t()) :: {:ok, pid()} | {:error, term()}
  def connect(%Client{} = client) do
    case Socket.whereis(client) do
      nil ->
        DynamicSupervisor.start_child(Rockbox.SocketSupervisor, {Socket, client})

      pid ->
        {:ok, pid}
    end
  end

  @doc "Tear down the WebSocket connection for `client`."
  @spec disconnect(Client.t()) :: :ok
  def disconnect(%Client{} = client), do: Socket.stop(client)

  @doc "Shortcut for `Rockbox.Events.subscribe/1`."
  defdelegate subscribe(event), to: Rockbox.Events

  @doc "Shortcut for `Rockbox.Events.unsubscribe/1`."
  defdelegate unsubscribe(event), to: Rockbox.Events

  # ---------------------------------------------------------------------------
  # Plugins
  # ---------------------------------------------------------------------------

  @doc "Install a plugin module. See `Rockbox.Plugin` for the behaviour."
  @spec use_plugin(Client.t(), module()) :: :ok | {:error, term()}
  def use_plugin(%Client{} = client, plugin), do: Plugins.install(client, plugin)

  @doc "Uninstall a plugin by module or by name string."
  @spec unuse_plugin(module() | String.t()) :: :ok
  def unuse_plugin(plugin), do: Plugins.uninstall(plugin)

  @doc "List currently installed plugins."
  @spec installed_plugins() :: [Plugins.entry()]
  def installed_plugins, do: Plugins.list()

  # ---------------------------------------------------------------------------
  # Raw GraphQL escape hatch
  # ---------------------------------------------------------------------------

  @doc """
  Run a raw GraphQL query or mutation. Variables can be a map or keyword list
  (snake_case keys are converted to camelCase). Returns `{:ok, data}`.

      Rockbox.query(client,
        \"\"\"
        query Album($id: String!) { album(id: $id) { id title artist } }
        \"\"\",
        id: "abc-123"
      )
  """
  @spec query(Client.t(), String.t(), map() | keyword() | nil) ::
          {:ok, map()} | {:error, Exception.t()}
  def query(%Client{} = client, gql, variables \\ nil),
    do: Transport.execute(client, gql, variables)

  @doc "Same as `query/3` but raises on error."
  @spec query!(Client.t(), String.t(), map() | keyword() | nil) :: map()
  def query!(%Client{} = client, gql, variables \\ nil),
    do: Transport.execute!(client, gql, variables)

  # ---------------------------------------------------------------------------
  # Misc helpers
  # ---------------------------------------------------------------------------

  @doc """
  Format a millisecond duration as `M:SS`.

      iex> Rockbox.format_ms(75_000)
      "1:15"
  """
  @spec format_ms(integer()) :: String.t()
  def format_ms(ms) when is_integer(ms) and ms >= 0 do
    total = div(ms, 1000)
    minutes = div(total, 60)
    seconds = rem(total, 60)
    "#{minutes}:#{seconds |> Integer.to_string() |> String.pad_leading(2, "0")}"
  end

  def format_ms(_), do: "0:00"
end
