# 12 — Plugin: sleep timer
#
# Demonstrates the Rockbox.Plugin behaviour. Stops playback after N minutes;
# cancels itself if the user already stopped playback manually.
#
#   mix run --no-halt examples/12_plugin_sleep_timer.exs        # default 30 min
#   mix run --no-halt examples/12_plugin_sleep_timer.exs 5      # 5 min

Code.require_file("_helper.exs", __DIR__)

defmodule SleepTimer do
  @behaviour Rockbox.Plugin
  use GenServer
  require Logger

  @impl Rockbox.Plugin
  def name, do: "sleep-timer"
  @impl Rockbox.Plugin
  def version, do: "1.0.0"
  @impl Rockbox.Plugin
  def description, do: "Stop playback after N minutes"

  @impl Rockbox.Plugin
  def install(ctx) do
    minutes = Application.get_env(:examples, :sleep_minutes, 30)
    {:ok, pid} = GenServer.start_link(__MODULE__, {ctx.client, minutes})
    {:ok, %{pid: pid}}
  end

  @impl Rockbox.Plugin
  def uninstall(%{pid: pid}) do
    if Process.alive?(pid), do: GenServer.stop(pid, :normal)
    :ok
  end

  # GenServer callbacks
  @impl GenServer
  def init({client, minutes}) do
    Rockbox.Events.subscribe(:status_changed)
    Process.send_after(self(), :fire, minutes * 60_000)
    fire_at = DateTime.add(DateTime.utc_now(), minutes * 60, :second)
    IO.puts("💤 Sleep timer armed — will stop playback at #{fire_at}")
    {:ok, %{client: client}}
  end

  @impl GenServer
  def handle_info(:fire, %{client: client} = state) do
    IO.puts("💤 Time's up — stopping playback.")
    Rockbox.Playback.stop(client)
    {:stop, :normal, state}
  end

  def handle_info({:rockbox, :status_changed, :stopped}, state) do
    IO.puts("💤 Playback stopped manually — sleep timer cancelled.")
    {:stop, :normal, state}
  end

  def handle_info(_other, state), do: {:noreply, state}
end

minutes =
  case System.argv() do
    [m | _] -> String.to_integer(m)
    _ -> 30
  end

Application.put_env(:examples, :sleep_minutes, minutes)

client = Examples.Helper.client()
{:ok, _} = Rockbox.connect(client)
:ok = Rockbox.use_plugin(client, SleepTimer)

IO.puts("Plugin installed. Press Ctrl+C twice to exit.")
Process.sleep(:infinity)
