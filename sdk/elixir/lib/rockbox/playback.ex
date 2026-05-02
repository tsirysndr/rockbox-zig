defmodule Rockbox.Playback do
  @moduledoc """
  Transport controls and play helpers.

      Rockbox.new()
      |> Rockbox.Playback.play_album("album-id", shuffle: true)

  Most functions return `:ok | {:error, exception}`. A `!` variant raises on
  error. Functions that return data come in `name/1 → {:ok, value}` and
  `name!/1 → value` pairs.
  """

  alias Rockbox.{Client, Track, Transport, Types, Util}

  @track_fields ~S"""
    fragment TrackFields on Track {
      id title artist album genre disc trackString yearString
      composer comment albumArtist grouping
      discnum tracknum layer year bitrate frequency
      filesize length elapsed path
      albumId artistId genreId albumArt
    }
  """

  # ---------------------------------------------------------------------------
  # State
  # ---------------------------------------------------------------------------

  @doc """
  Current playback status as an atom (`:stopped | :playing | :paused`).

      iex> Rockbox.Playback.status(client)
      {:ok, :playing}
  """
  @spec status(Client.t()) :: {:ok, Types.playback_status()} | {:error, Exception.t()}
  def status(client) do
    case Transport.execute(client, "query PlaybackStatus { status }") do
      {:ok, %{"status" => raw}} -> {:ok, Types.playback_status(raw)}
      err -> err
    end
  end

  @spec status!(Client.t()) :: Types.playback_status()
  def status!(client), do: bang(status(client))

  @doc "Raw integer playback status (matches the firmware enum)."
  @spec raw_status(Client.t()) :: {:ok, integer()} | {:error, Exception.t()}
  def raw_status(client) do
    case Transport.execute(client, "query PlaybackStatus { status }") do
      {:ok, %{"status" => raw}} -> {:ok, raw}
      err -> err
    end
  end

  @doc "The currently playing track, or `{:ok, nil}` when stopped."
  @spec current_track(Client.t()) :: {:ok, Track.t() | nil} | {:error, Exception.t()}
  def current_track(client) do
    query = @track_fields <> "query CurrentTrack { currentTrack { ...TrackFields } }"

    case Transport.execute(client, query) do
      {:ok, %{"currentTrack" => raw}} -> {:ok, Util.to_struct(Track, raw)}
      err -> err
    end
  end

  @spec current_track!(Client.t()) :: Track.t() | nil
  def current_track!(client), do: bang(current_track(client))

  @doc "The next track in the queue, or `{:ok, nil}` if there is none."
  @spec next_track(Client.t()) :: {:ok, Track.t() | nil} | {:error, Exception.t()}
  def next_track(client) do
    query = @track_fields <> "query NextTrack { nextTrack { ...TrackFields } }"

    case Transport.execute(client, query) do
      {:ok, %{"nextTrack" => raw}} -> {:ok, Util.to_struct(Track, raw)}
      err -> err
    end
  end

  @spec next_track!(Client.t()) :: Track.t() | nil
  def next_track!(client), do: bang(next_track(client))

  @doc "Byte offset into the currently playing file."
  @spec file_position(Client.t()) :: {:ok, integer()} | {:error, Exception.t()}
  def file_position(client) do
    case Transport.execute(client, "query FilePosition { getFilePosition }") do
      {:ok, %{"getFilePosition" => pos}} -> {:ok, pos}
      err -> err
    end
  end

  # ---------------------------------------------------------------------------
  # Transport controls
  # ---------------------------------------------------------------------------

  @doc "Resume playback from the queued position."
  @spec play(Client.t(), keyword()) :: :ok | {:error, Exception.t()}
  def play(client, opts \\ []) do
    elapsed = Keyword.get(opts, :elapsed, 0)
    offset = Keyword.get(opts, :offset, 0)

    void(
      Transport.execute(
        client,
        "mutation Play($elapsed: Long!, $offset: Long!) { play(elapsed: $elapsed, offset: $offset) }",
        %{elapsed: elapsed, offset: offset}
      )
    )
  end

  @spec play!(Client.t(), keyword()) :: :ok
  def play!(client, opts \\ []), do: bang(play(client, opts))

  for {fun, mutation} <- [
        pause: "mutation Pause { pause }",
        resume: "mutation Resume { resume }",
        next: "mutation Next { next }",
        previous: "mutation Previous { previous }",
        stop: "mutation Stop { hardStop }",
        flush_and_reload: "mutation FlushReload { flushAndReloadTracks }"
      ] do
    @doc "Run the corresponding mutation."
    @spec unquote(fun)(Client.t()) :: :ok | {:error, Exception.t()}
    def unquote(fun)(client), do: void(Transport.execute(client, unquote(mutation)))

    bang_name = String.to_atom("#{fun}!")
    @doc false
    @spec unquote(bang_name)(Client.t()) :: :ok
    def unquote(bang_name)(client), do: bang(unquote(fun)(client))
  end

  @doc "Seek to an absolute position in milliseconds."
  @spec seek(Client.t(), integer()) :: :ok | {:error, Exception.t()}
  def seek(client, position_ms) when is_integer(position_ms) do
    void(
      Transport.execute(
        client,
        "mutation Seek($newTime: Int!) { fastForwardRewind(newTime: $newTime) }",
        %{new_time: position_ms}
      )
    )
  end

  @spec seek!(Client.t(), integer()) :: :ok
  def seek!(client, ms), do: bang(seek(client, ms))

  # ---------------------------------------------------------------------------
  # Play helpers (single-call shortcuts)
  # ---------------------------------------------------------------------------

  @doc "Play a single file by absolute path."
  @spec play_track(Client.t(), String.t()) :: :ok | {:error, Exception.t()}
  def play_track(client, path) do
    void(
      Transport.execute(
        client,
        "mutation PlayTrack($path: String!) { playTrack(path: $path) }",
        %{path: path}
      )
    )
  end

  @spec play_track!(Client.t(), String.t()) :: :ok
  def play_track!(client, path), do: bang(play_track(client, path))

  @doc """
  Play all tracks from an album.

  Options: `:shuffle`, `:position` (start track index).
  """
  @spec play_album(Client.t(), String.t(), keyword()) :: :ok | {:error, Exception.t()}
  def play_album(client, album_id, opts \\ []) do
    vars = Map.merge(%{album_id: album_id}, Map.new(opts))

    void(
      Transport.execute(
        client,
        "mutation PlayAlbum($albumId: String!, $shuffle: Boolean, $position: Int) { playAlbum(albumId: $albumId, shuffle: $shuffle, position: $position) }",
        vars
      )
    )
  end

  @spec play_album!(Client.t(), String.t(), keyword()) :: :ok
  def play_album!(client, id, opts \\ []), do: bang(play_album(client, id, opts))

  @doc "Play all tracks by an artist. Options: `:shuffle`, `:position`."
  @spec play_artist(Client.t(), String.t(), keyword()) :: :ok | {:error, Exception.t()}
  def play_artist(client, artist_id, opts \\ []) do
    vars = Map.merge(%{artist_id: artist_id}, Map.new(opts))

    void(
      Transport.execute(
        client,
        "mutation PlayArtist($artistId: String!, $shuffle: Boolean, $position: Int) { playArtistTracks(artistId: $artistId, shuffle: $shuffle, position: $position) }",
        vars
      )
    )
  end

  @spec play_artist!(Client.t(), String.t(), keyword()) :: :ok
  def play_artist!(client, id, opts \\ []), do: bang(play_artist(client, id, opts))

  @doc "Play a saved playlist by id. Options: `:shuffle`, `:position`."
  @spec play_playlist(Client.t(), String.t(), keyword()) :: :ok | {:error, Exception.t()}
  def play_playlist(client, playlist_id, opts \\ []) do
    vars = Map.merge(%{playlist_id: playlist_id}, Map.new(opts))

    void(
      Transport.execute(
        client,
        "mutation PlayPlaylist($playlistId: String!, $shuffle: Boolean, $position: Int) { playPlaylist(playlistId: $playlistId, shuffle: $shuffle, position: $position) }",
        vars
      )
    )
  end

  @spec play_playlist!(Client.t(), String.t(), keyword()) :: :ok
  def play_playlist!(client, id, opts \\ []), do: bang(play_playlist(client, id, opts))

  @doc "Play every file under a directory. Options: `:recurse`, `:shuffle`, `:position`."
  @spec play_directory(Client.t(), String.t(), keyword()) :: :ok | {:error, Exception.t()}
  def play_directory(client, path, opts \\ []) do
    vars = Map.merge(%{path: path}, Map.new(opts))

    void(
      Transport.execute(
        client,
        "mutation PlayDirectory($path: String!, $recurse: Boolean, $shuffle: Boolean, $position: Int) { playDirectory(path: $path, recurse: $recurse, shuffle: $shuffle, position: $position) }",
        vars
      )
    )
  end

  @spec play_directory!(Client.t(), String.t(), keyword()) :: :ok
  def play_directory!(client, path, opts \\ []), do: bang(play_directory(client, path, opts))

  @doc "Play the user's liked tracks."
  @spec play_liked_tracks(Client.t(), keyword()) :: :ok | {:error, Exception.t()}
  def play_liked_tracks(client, opts \\ []) do
    void(
      Transport.execute(
        client,
        "mutation PlayLikedTracks($shuffle: Boolean, $position: Int) { playLikedTracks(shuffle: $shuffle, position: $position) }",
        Map.new(opts)
      )
    )
  end

  @spec play_liked_tracks!(Client.t(), keyword()) :: :ok
  def play_liked_tracks!(client, opts \\ []), do: bang(play_liked_tracks(client, opts))

  @doc "Play the entire library — typically with `shuffle: true`."
  @spec play_all_tracks(Client.t(), keyword()) :: :ok | {:error, Exception.t()}
  def play_all_tracks(client, opts \\ []) do
    void(
      Transport.execute(
        client,
        "mutation PlayAllTracks($shuffle: Boolean, $position: Int) { playAllTracks(shuffle: $shuffle, position: $position) }",
        Map.new(opts)
      )
    )
  end

  @spec play_all_tracks!(Client.t(), keyword()) :: :ok
  def play_all_tracks!(client, opts \\ []), do: bang(play_all_tracks(client, opts))

  # ---------------------------------------------------------------------------
  # Internal
  # ---------------------------------------------------------------------------

  defp void({:ok, _}), do: :ok
  defp void(err), do: err

  defp bang({:ok, value}), do: value
  defp bang(:ok), do: :ok
  defp bang({:error, exception}), do: raise(exception)
end
