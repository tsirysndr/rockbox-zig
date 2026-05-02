defmodule Rockbox.Plugins do
  @moduledoc """
  Registry of installed `Rockbox.Plugin` modules. You normally interact with
  it through `Rockbox.use_plugin/2`, `Rockbox.unuse_plugin/2`, and
  `Rockbox.installed_plugins/0`.
  """

  use GenServer

  alias Rockbox.{Client, Plugin}

  @type entry :: %{module: module(), state: term(), client: Client.t()}

  # ---------------------------------------------------------------------------
  # Public API
  # ---------------------------------------------------------------------------

  @spec start_link(term()) :: GenServer.on_start()
  def start_link(_), do: GenServer.start_link(__MODULE__, %{}, name: __MODULE__)

  @spec install(Client.t(), module()) :: :ok | {:error, term()}
  def install(client, plugin), do: GenServer.call(__MODULE__, {:install, client, plugin})

  @spec uninstall(module() | String.t()) :: :ok
  def uninstall(name_or_module), do: GenServer.call(__MODULE__, {:uninstall, name_or_module})

  @spec list() :: [entry()]
  def list, do: GenServer.call(__MODULE__, :list)

  # ---------------------------------------------------------------------------
  # GenServer
  # ---------------------------------------------------------------------------

  @impl true
  def init(_), do: {:ok, %{}}

  @impl true
  def handle_call({:install, client, plugin}, _from, state) do
    name = plugin.name()

    if Map.has_key?(state, name) do
      {:reply, {:error, :already_installed}, state}
    else
      ctx = %{client: client}

      case plugin.install(ctx) do
        {:ok, plugin_state} ->
          entry = %{module: plugin, state: plugin_state, client: client}
          {:reply, :ok, Map.put(state, name, entry)}

        :ok ->
          entry = %{module: plugin, state: nil, client: client}
          {:reply, :ok, Map.put(state, name, entry)}

        {:error, _} = err ->
          {:reply, err, state}
      end
    end
  end

  def handle_call({:uninstall, key}, _from, state) do
    name = if is_atom(key), do: try_name(key, key), else: key

    case Map.pop(state, name) do
      {nil, state} ->
        {:reply, :ok, state}

      {%{module: mod, state: pstate}, state} ->
        Plugin.uninstall(mod, pstate)
        {:reply, :ok, state}
    end
  end

  def handle_call(:list, _from, state) do
    {:reply, Map.values(state), state}
  end

  defp try_name(plugin, fallback) do
    if function_exported?(plugin, :name, 0), do: plugin.name(), else: to_string(fallback)
  end
end
