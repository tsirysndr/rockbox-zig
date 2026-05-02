defmodule Rockbox.Library do
  @moduledoc """
  Browse and manage the music library — albums, artists, tracks, likes,
  search, and rescan.
  """

  alias Rockbox.{Album, Artist, Client, SearchResults, Track, Transport, Util}

  @track_fields ~S"""
    fragment TrackFields on Track {
      id title artist album genre disc trackString yearString
      composer comment albumArtist grouping
      discnum tracknum layer year bitrate frequency
      filesize length elapsed path
      albumId artistId genreId albumArt
    }
  """

  @album_fields ~S"""
    fragment AlbumFields on Album {
      id title artist year yearString albumArt md5 artistId copyrightMessage
    }
  """

  @artist_fields ~S"""
    fragment ArtistFields on Artist {
      id name bio image
    }
  """

  # ---------------------------------------------------------------------------
  # Albums
  # ---------------------------------------------------------------------------

  @doc "List every album. Each album's `:tracks` are stub records (id/title/path/length/album_art)."
  @spec albums(Client.t()) :: {:ok, [Album.t()]} | {:error, Exception.t()}
  def albums(client) do
    query =
      @album_fields <>
        "query Albums { albums { ...AlbumFields tracks { id title path length albumArt } } }"

    with {:ok, %{"albums" => list}} <- Transport.execute(client, query) do
      {:ok, Enum.map(list, &album_with_tracks/1)}
    end
  end

  @spec albums!(Client.t()) :: [Album.t()]
  def albums!(client), do: bang(albums(client))

  @doc "Get a single album with full track info, or `{:ok, nil}`."
  @spec album(Client.t(), String.t()) :: {:ok, Album.t() | nil} | {:error, Exception.t()}
  def album(client, id) do
    query =
      @track_fields <>
        @album_fields <>
        "query Album($id: String!) { album(id: $id) { ...AlbumFields tracks { ...TrackFields } } }"

    with {:ok, %{"album" => raw}} <- Transport.execute(client, query, %{id: id}) do
      {:ok, album_with_tracks(raw)}
    end
  end

  @spec album!(Client.t(), String.t()) :: Album.t() | nil
  def album!(client, id), do: bang(album(client, id))

  @doc "List albums the user has liked."
  @spec liked_albums(Client.t()) :: {:ok, [Album.t()]} | {:error, Exception.t()}
  def liked_albums(client) do
    query = @album_fields <> "query LikedAlbums { likedAlbums { ...AlbumFields } }"

    with {:ok, %{"likedAlbums" => list}} <- Transport.execute(client, query) do
      {:ok, Util.to_struct_list(Album, list)}
    end
  end

  @spec like_album(Client.t(), String.t()) :: :ok | {:error, Exception.t()}
  def like_album(client, id),
    do:
      void(
        Transport.execute(client, "mutation LikeAlbum($id: String!) { likeAlbum(id: $id) }", %{
          id: id
        })
      )

  @spec unlike_album(Client.t(), String.t()) :: :ok | {:error, Exception.t()}
  def unlike_album(client, id),
    do:
      void(
        Transport.execute(
          client,
          "mutation UnlikeAlbum($id: String!) { unlikeAlbum(id: $id) }",
          %{
            id: id
          }
        )
      )

  # ---------------------------------------------------------------------------
  # Artists
  # ---------------------------------------------------------------------------

  @doc "List every artist with shallow album info."
  @spec artists(Client.t()) :: {:ok, [Artist.t()]} | {:error, Exception.t()}
  def artists(client) do
    query =
      @artist_fields <>
        "query Artists { artists { ...ArtistFields albums { id title albumArt year } } }"

    with {:ok, %{"artists" => list}} <- Transport.execute(client, query) do
      {:ok, Enum.map(list, &artist_with_albums/1)}
    end
  end

  @spec artists!(Client.t()) :: [Artist.t()]
  def artists!(client), do: bang(artists(client))

  @doc "Get a single artist with their albums and tracks."
  @spec artist(Client.t(), String.t()) :: {:ok, Artist.t() | nil} | {:error, Exception.t()}
  def artist(client, id) do
    query =
      @artist_fields <>
        @track_fields <>
        """
        query Artist($id: String!) {
          artist(id: $id) {
            ...ArtistFields
            albums { id title albumArt year yearString md5 artistId tracks { id title path length } }
            tracks { ...TrackFields }
          }
        }
        """

    with {:ok, %{"artist" => raw}} <- Transport.execute(client, query, %{id: id}) do
      {:ok, artist_with_albums(raw)}
    end
  end

  @spec artist!(Client.t(), String.t()) :: Artist.t() | nil
  def artist!(client, id), do: bang(artist(client, id))

  # ---------------------------------------------------------------------------
  # Tracks
  # ---------------------------------------------------------------------------

  @doc "List every track."
  @spec tracks(Client.t()) :: {:ok, [Track.t()]} | {:error, Exception.t()}
  def tracks(client) do
    query = @track_fields <> "query Tracks { tracks { ...TrackFields } }"

    with {:ok, %{"tracks" => list}} <- Transport.execute(client, query) do
      {:ok, Util.to_struct_list(Track, list)}
    end
  end

  @spec tracks!(Client.t()) :: [Track.t()]
  def tracks!(client), do: bang(tracks(client))

  @doc "Get a single track by id, or `{:ok, nil}`."
  @spec track(Client.t(), String.t()) :: {:ok, Track.t() | nil} | {:error, Exception.t()}
  def track(client, id) do
    query = @track_fields <> "query Track($id: String!) { track(id: $id) { ...TrackFields } }"

    with {:ok, %{"track" => raw}} <- Transport.execute(client, query, %{id: id}) do
      {:ok, Util.to_struct(Track, raw)}
    end
  end

  @spec track!(Client.t(), String.t()) :: Track.t() | nil
  def track!(client, id), do: bang(track(client, id))

  @doc "List tracks the user has liked."
  @spec liked_tracks(Client.t()) :: {:ok, [Track.t()]} | {:error, Exception.t()}
  def liked_tracks(client) do
    query = @track_fields <> "query LikedTracks { likedTracks { ...TrackFields } }"

    with {:ok, %{"likedTracks" => list}} <- Transport.execute(client, query) do
      {:ok, Util.to_struct_list(Track, list)}
    end
  end

  @spec like_track(Client.t(), String.t()) :: :ok | {:error, Exception.t()}
  def like_track(client, id),
    do:
      void(
        Transport.execute(client, "mutation LikeTrack($id: String!) { likeTrack(id: $id) }", %{
          id: id
        })
      )

  @spec unlike_track(Client.t(), String.t()) :: :ok | {:error, Exception.t()}
  def unlike_track(client, id),
    do:
      void(
        Transport.execute(
          client,
          "mutation UnlikeTrack($id: String!) { unlikeTrack(id: $id) }",
          %{
            id: id
          }
        )
      )

  # ---------------------------------------------------------------------------
  # Search
  # ---------------------------------------------------------------------------

  @doc "Full-text search across artists, albums and tracks."
  @spec search(Client.t(), String.t()) :: {:ok, SearchResults.t()} | {:error, Exception.t()}
  def search(client, term) do
    query =
      @track_fields <>
        @album_fields <>
        @artist_fields <>
        """
        query Search($term: String!) {
          search(term: $term) {
            artists { ...ArtistFields }
            albums { ...AlbumFields }
            tracks { ...TrackFields }
            likedTracks { ...TrackFields }
            likedAlbums { ...AlbumFields }
          }
        }
        """

    with {:ok, %{"search" => raw}} <- Transport.execute(client, query, %{term: term}) do
      atomized = Util.atomize(raw)

      {:ok,
       %SearchResults{
         artists: Util.to_struct_list(Artist, atomized.artists),
         albums: Util.to_struct_list(Album, atomized.albums),
         tracks: Util.to_struct_list(Track, atomized.tracks),
         liked_tracks: Util.to_struct_list(Track, atomized.liked_tracks),
         liked_albums: Util.to_struct_list(Album, atomized.liked_albums)
       }}
    end
  end

  @spec search!(Client.t(), String.t()) :: SearchResults.t()
  def search!(client, term), do: bang(search(client, term))

  # ---------------------------------------------------------------------------
  # Library management
  # ---------------------------------------------------------------------------

  @doc "Trigger a full rescan of the configured `music_dir`."
  @spec scan(Client.t()) :: :ok | {:error, Exception.t()}
  def scan(client),
    do: void(Transport.execute(client, "mutation ScanLibrary { scanLibrary }"))

  # ---------------------------------------------------------------------------
  # Internal
  # ---------------------------------------------------------------------------

  defp album_with_tracks(nil), do: nil

  defp album_with_tracks(raw) do
    atomized = Util.atomize(raw)
    base = Util.to_struct(Album, raw)
    %{base | tracks: Util.to_struct_list(Track, Map.get(atomized, :tracks, []))}
  end

  defp artist_with_albums(nil), do: nil

  defp artist_with_albums(raw) do
    atomized = Util.atomize(raw)
    base = Util.to_struct(Artist, raw)

    albums =
      atomized
      |> Map.get(:albums, [])
      |> Enum.map(&album_with_tracks/1)

    tracks = Util.to_struct_list(Track, Map.get(atomized, :tracks, []))
    %{base | albums: albums, tracks: tracks}
  end

  defp void({:ok, _}), do: :ok
  defp void(err), do: err

  defp bang({:ok, value}), do: value
  defp bang(:ok), do: :ok
  defp bang({:error, exception}), do: raise(exception)
end
