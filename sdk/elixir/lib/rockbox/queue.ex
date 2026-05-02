defmodule Rockbox.Queue do
  @moduledoc """
  The live playback queue (called *playlist* in the GraphQL schema). For
  persistent named collections see `Rockbox.SavedPlaylists`.

      Rockbox.new()
      |> Rockbox.Queue.insert_tracks(["/Music/a.mp3", "/Music/b.mp3"], :next)
  """

  alias Rockbox.{Client, Playlist, Track, Transport, Types, Util}

  @track_fields ~S"""
    fragment TrackFields on Track {
      id title artist album genre disc trackString yearString
      composer comment albumArtist grouping
      discnum tracknum layer year bitrate frequency
      filesize length elapsed path
      albumId artistId genreId albumArt
    }
  """

  @doc "Snapshot of the active queue."
  @spec current(Client.t()) :: {:ok, Playlist.t()} | {:error, Exception.t()}
  def current(client) do
    query =
      @track_fields <>
        """
        query CurrentPlaylist {
          playlistGetCurrent {
            amount index maxPlaylistSize firstIndex
            lastInsertPos seed lastShuffledStart
            tracks { ...TrackFields }
          }
        }
        """

    with {:ok, %{"playlistGetCurrent" => raw}} <- Transport.execute(client, query) do
      atomized = Util.atomize(raw)
      base = Util.to_struct(Playlist, raw)
      {:ok, %{base | tracks: Util.to_struct_list(Track, Map.get(atomized, :tracks, []))}}
    end
  end

  @spec current!(Client.t()) :: Playlist.t()
  def current!(client), do: bang(current(client))

  @doc "Number of tracks currently queued."
  @spec amount(Client.t()) :: {:ok, integer()} | {:error, Exception.t()}
  def amount(client) do
    case Transport.execute(client, "query PlaylistAmount { playlistAmount }") do
      {:ok, %{"playlistAmount" => n}} -> {:ok, n}
      err -> err
    end
  end

  # ---------------------------------------------------------------------------
  # Insert / remove
  # ---------------------------------------------------------------------------

  @doc """
  Insert one or more file paths (or track ids) into the queue.

  `position` may be an atom (`:next`, `:after_current`, `:last`, `:first`) or
  the matching integer. `playlist_id` is optional — omit to target the active
  queue.

      Rockbox.Queue.insert_tracks(client, ["/Music/a.mp3"], :next)
  """
  @spec insert_tracks(Client.t(), [String.t()], Types.insert_position(), String.t() | nil) ::
          :ok | {:error, Exception.t()}
  def insert_tracks(client, paths, position \\ :next, playlist_id \\ nil) do
    vars = %{playlist_id: playlist_id, position: Types.insert_position(position), tracks: paths}

    void(
      Transport.execute(
        client,
        "mutation InsertTracks($playlistId: String, $position: Int!, $tracks: [String!]!) { insertTracks(playlistId: $playlistId, position: $position, tracks: $tracks) }",
        vars
      )
    )
  end

  @doc "Insert every file under a directory into the queue."
  @spec insert_directory(Client.t(), String.t(), Types.insert_position(), String.t() | nil) ::
          :ok | {:error, Exception.t()}
  def insert_directory(client, directory, position \\ :last, playlist_id \\ nil) do
    vars = %{
      playlist_id: playlist_id,
      position: Types.insert_position(position),
      directory: directory
    }

    void(
      Transport.execute(
        client,
        "mutation InsertDirectory($playlistId: String, $position: Int!, $directory: String!) { insertDirectory(playlistId: $playlistId, position: $position, directory: $directory) }",
        vars
      )
    )
  end

  @doc "Insert every track from an album into the queue."
  @spec insert_album(Client.t(), String.t(), Types.insert_position()) ::
          :ok | {:error, Exception.t()}
  def insert_album(client, album_id, position \\ :last) do
    vars = %{album_id: album_id, position: Types.insert_position(position)}

    void(
      Transport.execute(
        client,
        "mutation InsertAlbum($albumId: String!, $position: Int!) { insertAlbum(albumId: $albumId, position: $position) }",
        vars
      )
    )
  end

  @doc "Remove the track at the given 0-based queue index."
  @spec remove_track(Client.t(), non_neg_integer()) :: :ok | {:error, Exception.t()}
  def remove_track(client, index) do
    void(
      Transport.execute(
        client,
        "mutation RemoveTrack($index: Int!) { playlistRemoveTrack(index: $index) }",
        %{index: index}
      )
    )
  end

  @doc "Empty the queue."
  @spec clear(Client.t()) :: :ok | {:error, Exception.t()}
  def clear(client),
    do: void(Transport.execute(client, "mutation ClearPlaylist { playlistRemoveAllTracks }"))

  @doc "Reshuffle the queue in place."
  @spec shuffle(Client.t()) :: :ok | {:error, Exception.t()}
  def shuffle(client),
    do: void(Transport.execute(client, "mutation ShufflePlaylist { shufflePlaylist }"))

  @doc "Create a new temporary queue (replaces the current one) and start playing."
  @spec create(Client.t(), String.t(), [String.t()]) :: :ok | {:error, Exception.t()}
  def create(client, name, tracks) do
    void(
      Transport.execute(
        client,
        "mutation CreatePlaylist($name: String!, $tracks: [String!]!) { playlistCreate(name: $name, tracks: $tracks) }",
        %{name: name, tracks: tracks}
      )
    )
  end

  @doc "Begin playback of the current queue. Options: `:start_index`, `:elapsed`, `:offset`."
  @spec start(Client.t(), keyword()) :: :ok | {:error, Exception.t()}
  def start(client, opts \\ []) do
    void(
      Transport.execute(
        client,
        "mutation PlaylistStart($startIndex: Int, $elapsed: Int, $offset: Int) { playlistStart(startIndex: $startIndex, elapsed: $elapsed, offset: $offset) }",
        Map.new(opts)
      )
    )
  end

  @doc "Resume the queue from where playback was last stopped."
  @spec resume(Client.t()) :: :ok | {:error, Exception.t()}
  def resume(client),
    do: void(Transport.execute(client, "mutation PlaylistResume { playlistResume }"))

  # ---------------------------------------------------------------------------
  # Internal
  # ---------------------------------------------------------------------------

  defp void({:ok, _}), do: :ok
  defp void(err), do: err

  defp bang({:ok, value}), do: value
  defp bang(:ok), do: :ok
  defp bang({:error, exception}), do: raise(exception)
end
