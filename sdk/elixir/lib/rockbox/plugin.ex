defmodule Rockbox.Plugin do
  @moduledoc """
  Behaviour for Rockbox plugins. A plugin is a regular module implementing
  `install/1` (and optionally `uninstall/1`).

      defmodule MyApp.LastFmScrobbler do
        @behaviour Rockbox.Plugin

        @impl true
        def name,    do: "lastfm-scrobbler"
        @impl true
        def version, do: "1.0.0"
        @impl true
        def description, do: "Scrobble played tracks to Last.fm"

        @impl true
        def install(ctx) do
          # Subscribe like any other process — events arrive as messages.
          Rockbox.Events.subscribe(:track_changed)
          {:ok, %{client: ctx.client, started_at: System.monotonic_time(:millisecond)}}
        end

        @impl true
        def uninstall(_state), do: :ok
      end

  Install with `Rockbox.use_plugin(client, MyApp.LastFmScrobbler)`. Plugins
  receive a `t:context/0` map containing the client they were installed with.

  Heavy plugins (those that need their own process) should spawn it inside
  `install/1` and store its pid in their state — `uninstall/1` will be called
  with that state and can shut the process down.
  """

  @type context :: %{client: Rockbox.Client.t()}
  @type state :: term()

  @callback name() :: String.t()
  @callback version() :: String.t()
  @callback description() :: String.t()
  @callback install(context()) :: {:ok, state()} | :ok | {:error, term()}
  @callback uninstall(state()) :: :ok | {:error, term()}

  @optional_callbacks description: 0, uninstall: 1

  @doc false
  def description(plugin) do
    if function_exported?(plugin, :description, 0), do: plugin.description(), else: nil
  end

  @doc false
  def uninstall(plugin, state) do
    if function_exported?(plugin, :uninstall, 1), do: plugin.uninstall(state), else: :ok
  end
end
