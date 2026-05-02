defmodule Rockbox.Socket do
  @moduledoc """
  GenServer that manages the WebSocket connection to rockboxd, speaks the
  `graphql-transport-ws` subprotocol, and forwards incoming events to
  subscribers via `Rockbox.Events.broadcast/2`.

  You normally don't interact with this module directly — call
  `Rockbox.connect/1` to start it and `Rockbox.disconnect/1` to stop it.

  Reconnects automatically with exponential backoff (capped at 30s).
  """

  use GenServer
  require Logger

  alias Rockbox.{Client, Events, Playlist, Track, Types, Util}

  @reconnect_min 1_000
  @reconnect_max 30_000

  # Subscriptions we always activate once the WS is connected.
  @subscriptions %{
    "track" => ~S"""
    subscription CurrentlyPlaying {
      currentlyPlayingSong {
        id title artist album albumArt albumId artistId path length elapsed
      }
    }
    """,
    "status" => ~S"subscription PlaybackStatus { playbackStatus { status } }",
    "playlist" => ~S"""
    subscription PlaylistChanged {
      playlistChanged {
        amount index maxPlaylistSize firstIndex lastInsertPos seed lastShuffledStart
        tracks { id title artist album path length albumArt }
      }
    }
    """
  }

  defstruct [
    :client,
    :conn,
    :websocket,
    :ref,
    :host,
    :port,
    :path,
    :scheme,
    reconnect_attempt: 0,
    buffer: [],
    state: :disconnected
  ]

  # ---------------------------------------------------------------------------
  # Public API
  # ---------------------------------------------------------------------------

  @doc "Start (or fetch) the socket for `client`. Returns `{:ok, pid}`."
  @spec start_link(Client.t()) :: {:ok, pid()} | {:error, term()}
  def start_link(%Client{} = client) do
    GenServer.start_link(__MODULE__, client, name: via(client))
  end

  @doc "Look up the running socket pid for `client`, if any."
  @spec whereis(Client.t()) :: pid() | nil
  def whereis(%Client{ws_url: url}) do
    case Registry.lookup(Rockbox.Sockets, url) do
      [{pid, _}] -> pid
      _ -> nil
    end
  end

  @doc "Stop the socket for `client`, if it's running."
  @spec stop(Client.t()) :: :ok
  def stop(%Client{} = client) do
    case whereis(client) do
      nil -> :ok
      pid -> GenServer.stop(pid, :normal)
    end
  end

  defp via(%Client{ws_url: url}), do: {:via, Registry, {Rockbox.Sockets, url}}

  # ---------------------------------------------------------------------------
  # GenServer callbacks
  # ---------------------------------------------------------------------------

  @impl true
  def init(client) do
    %URI{scheme: scheme_str, host: host, port: port, path: path} = URI.parse(client.ws_url)

    scheme =
      case scheme_str do
        "wss" -> :https
        _ -> :http
      end

    state = %__MODULE__{
      client: client,
      host: host,
      port: port || default_port(scheme),
      path: path || "/graphql",
      scheme: scheme
    }

    send(self(), :connect)
    {:ok, state}
  end

  @impl true
  def handle_info(:connect, %__MODULE__{} = state) do
    case open_websocket(state) do
      {:ok, new_state} ->
        Events.broadcast(:ws_open, nil)
        new_state = send_init(new_state)
        new_state = subscribe_all(new_state)
        {:noreply, %{new_state | state: :ready, reconnect_attempt: 0}}

      {:error, reason} ->
        Logger.warning("[Rockbox.Socket] connect failed: #{inspect(reason)}")

        Events.broadcast(:ws_error, %Rockbox.NetworkError{
          message: "WebSocket connect failed",
          cause: reason
        })

        schedule_reconnect(state)
    end
  end

  def handle_info(message, %__MODULE__{conn: conn} = state) when not is_nil(conn) do
    case Mint.WebSocket.stream(conn, message) do
      {:ok, conn, responses} ->
        state = %{state | conn: conn}
        Enum.reduce(responses, {:noreply, state}, &handle_response/2)

      {:error, conn, reason, _responses} ->
        Logger.warning("[Rockbox.Socket] stream error: #{inspect(reason)}")

        Events.broadcast(:ws_error, %Rockbox.NetworkError{
          message: "WebSocket stream error",
          cause: reason
        })

        Events.broadcast(:ws_close, nil)
        schedule_reconnect(%{state | conn: conn, websocket: nil, state: :disconnected})

      :unknown ->
        {:noreply, state}
    end
  end

  def handle_info(_other, state), do: {:noreply, state}

  @impl true
  def terminate(_reason, %__MODULE__{conn: conn}) when not is_nil(conn) do
    Mint.HTTP.close(conn)
    Events.broadcast(:ws_close, nil)
    :ok
  end

  def terminate(_, _), do: :ok

  # ---------------------------------------------------------------------------
  # Internal — WebSocket plumbing
  # ---------------------------------------------------------------------------

  defp open_websocket(%__MODULE__{} = state) do
    with {:ok, conn} <-
           Mint.HTTP.connect(state.scheme, state.host, state.port, protocols: [:http1]),
         {:ok, conn, ref} <-
           Mint.WebSocket.upgrade(ws_scheme(state.scheme), conn, state.path, [
             {"sec-websocket-protocol", "graphql-transport-ws"}
           ]) do
      {:ok, %{state | conn: conn, ref: ref, state: :upgrading}}
    end
  end

  defp ws_scheme(:http), do: :ws
  defp ws_scheme(:https), do: :wss

  defp default_port(:http), do: 80
  defp default_port(:https), do: 443

  defp send_init(state), do: send_text(state, %{type: "connection_init", payload: %{}})

  defp subscribe_all(state) do
    Enum.reduce(@subscriptions, state, fn {id, query}, acc ->
      send_text(acc, %{id: id, type: "subscribe", payload: %{query: query}})
    end)
  end

  defp send_text(%__MODULE__{websocket: nil} = state, _msg), do: state

  defp send_text(%__MODULE__{conn: conn, websocket: ws, ref: ref} = state, msg) do
    json = Jason.encode!(msg)

    case Mint.WebSocket.encode(ws, {:text, json}) do
      {:ok, ws, data} ->
        case Mint.WebSocket.stream_request_body(conn, ref, data) do
          {:ok, conn} ->
            %{state | conn: conn, websocket: ws}

          {:error, conn, reason} ->
            Logger.warning("[Rockbox.Socket] send error: #{inspect(reason)}")
            %{state | conn: conn, websocket: ws}
        end

      {:error, ws, reason} ->
        Logger.warning("[Rockbox.Socket] encode error: #{inspect(reason)}")
        %{state | websocket: ws}
    end
  end

  defp handle_response({:status, ref, status}, {:noreply, %{ref: ref} = state}),
    do: {:noreply, %{state | buffer: [{:status, status} | state.buffer]}}

  defp handle_response({:headers, ref, headers}, {:noreply, %{ref: ref, conn: conn} = state}) do
    [{:status, status} | _] = state.buffer

    case Mint.WebSocket.new(conn, ref, status, headers) do
      {:ok, conn, ws} ->
        state = %{state | conn: conn, websocket: ws, buffer: [], state: :connected}
        state = send_init(state)
        state = subscribe_all(state)
        {:noreply, state}

      {:error, conn, reason} ->
        Logger.warning("[Rockbox.Socket] WS upgrade failed: #{inspect(reason)}")

        Events.broadcast(:ws_error, %Rockbox.NetworkError{
          message: "WebSocket upgrade failed",
          cause: reason
        })

        Mint.HTTP.close(conn)
        {:noreply, %{state | conn: nil, websocket: nil, state: :disconnected}}
    end
  end

  defp handle_response({:data, ref, data}, {:noreply, %{ref: ref, websocket: ws} = state})
       when not is_nil(ws) do
    case Mint.WebSocket.decode(ws, data) do
      {:ok, ws, frames} ->
        state = %{state | websocket: ws}
        state = Enum.reduce(frames, state, &handle_frame/2)
        {:noreply, state}

      {:error, ws, reason} ->
        Logger.warning("[Rockbox.Socket] decode error: #{inspect(reason)}")
        {:noreply, %{state | websocket: ws}}
    end
  end

  defp handle_response({:done, _ref}, acc), do: acc
  defp handle_response(_other, acc), do: acc

  defp handle_frame({:text, text}, state) do
    case Jason.decode(text) do
      {:ok, msg} -> dispatch_message(msg, state)
      _ -> state
    end
  end

  defp handle_frame({:ping, payload}, state), do: send_pong(state, payload)
  defp handle_frame({:close, _, _}, state), do: state
  defp handle_frame(_, state), do: state

  defp send_pong(%__MODULE__{conn: conn, websocket: ws, ref: ref} = state, payload) do
    case Mint.WebSocket.encode(ws, {:pong, payload || ""}) do
      {:ok, ws, data} ->
        case Mint.WebSocket.stream_request_body(conn, ref, data) do
          {:ok, conn} -> %{state | conn: conn, websocket: ws}
          _ -> %{state | websocket: ws}
        end

      _ ->
        state
    end
  end

  # graphql-ws protocol messages
  defp dispatch_message(%{"type" => "connection_ack"}, state), do: state

  defp dispatch_message(%{"type" => "ping"}, state),
    do: send_text(state, %{type: "pong"})

  defp dispatch_message(%{"type" => "pong"}, state), do: state

  defp dispatch_message(%{"type" => "next", "id" => id, "payload" => payload}, state) do
    handle_subscription_payload(id, payload)
    state
  end

  defp dispatch_message(%{"type" => "error", "id" => id, "payload" => payload}, state) do
    Logger.warning("[Rockbox.Socket] subscription #{id} error: #{inspect(payload)}")
    state
  end

  defp dispatch_message(%{"type" => "complete"}, state), do: state
  defp dispatch_message(_other, state), do: state

  defp handle_subscription_payload("track", %{"data" => %{"currentlyPlayingSong" => raw}})
       when not is_nil(raw),
       do: Events.broadcast(:track_changed, Util.to_struct(Track, raw))

  defp handle_subscription_payload("status", %{
         "data" => %{"playbackStatus" => %{"status" => raw}}
       }),
       do: Events.broadcast(:status_changed, Types.playback_status(raw))

  defp handle_subscription_payload("playlist", %{"data" => %{"playlistChanged" => raw}})
       when not is_nil(raw) do
    atomized = Util.atomize(raw)
    base = Util.to_struct(Playlist, raw)
    pl = %{base | tracks: Util.to_struct_list(Track, Map.get(atomized, :tracks, []))}
    Events.broadcast(:playlist_changed, pl)
  end

  defp handle_subscription_payload(_id, _payload), do: :ok

  # ---------------------------------------------------------------------------
  # Reconnect
  # ---------------------------------------------------------------------------

  defp schedule_reconnect(%__MODULE__{reconnect_attempt: n} = state) do
    delay = min(@reconnect_min * round(:math.pow(2, n)), @reconnect_max)
    Process.send_after(self(), :connect, delay)

    {:noreply,
     %{state | reconnect_attempt: n + 1, conn: nil, websocket: nil, state: :reconnecting}}
  end
end
