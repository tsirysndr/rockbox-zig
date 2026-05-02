defmodule Rockbox.Events do
  @moduledoc """
  Subscribe the calling process to real-time events emitted by rockboxd.

  Events arrive as plain Erlang messages so they integrate naturally with
  `receive` blocks and `GenServer.handle_info/2`:

      {:ok, _pid} = Rockbox.connect(client)
      :ok = Rockbox.Events.subscribe(:track_changed)

      receive do
        {:rockbox, :track_changed, %Rockbox.Track{} = track} ->
          IO.puts("Now playing: \#{track.title}")
      end

  ## Event names and payloads

  | event              | payload                                    |
  |--------------------|--------------------------------------------|
  | `:track_changed`   | `Rockbox.Track.t()`                        |
  | `:status_changed`  | `:stopped | :playing | :paused`            |
  | `:playlist_changed`| `Rockbox.Playlist.t()`                     |
  | `:ws_open`         | `nil`                                      |
  | `:ws_close`        | `nil`                                      |
  | `:ws_error`        | `Exception.t()`                            |

  Subscribers are auto-removed when their process exits, so you never need to
  manually clean up.
  """

  @typedoc "All event names emitted by the SDK."
  @type event ::
          :track_changed
          | :status_changed
          | :playlist_changed
          | :ws_open
          | :ws_close
          | :ws_error

  @typedoc "All event names plus `:all` for catch-all subscribers."
  @type subscription :: event() | :all

  @doc """
  Subscribe the calling process to one or more events.

  Pass a single atom (or `:all` for catch-all), or a list of atoms.
  """
  @spec subscribe(subscription() | [subscription()]) :: :ok
  def subscribe(event) when is_atom(event) do
    {:ok, _} = Registry.register(Rockbox.Subscribers, event, [])
    :ok
  end

  def subscribe(events) when is_list(events) do
    Enum.each(events, &subscribe/1)
  end

  @doc "Unsubscribe the calling process from an event."
  @spec unsubscribe(subscription()) :: :ok
  def unsubscribe(event) when is_atom(event) do
    Registry.unregister(Rockbox.Subscribers, event)
  end

  @doc false
  @spec broadcast(event(), term()) :: :ok
  def broadcast(event, payload) do
    deliver(event, payload)
    deliver(:all, {event, payload})
    :ok
  end

  defp deliver(key, payload) do
    Registry.dispatch(Rockbox.Subscribers, key, fn entries ->
      for {pid, _} <- entries do
        send(pid, message(key, payload))
      end
    end)
  end

  defp message(:all, {event, payload}), do: {:rockbox, event, payload}
  defp message(event, payload), do: {:rockbox, event, payload}
end
