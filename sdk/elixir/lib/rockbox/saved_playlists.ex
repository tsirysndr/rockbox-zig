defmodule Rockbox.SavedPlaylists do
  @moduledoc """
  Persistent named playlists, optionally grouped into folders.

      {:ok, pl} = Rockbox.SavedPlaylists.create(client,
        name: "Late Night Jazz",
        track_ids: ["t1", "t2", "t3"]
      )
  """

  alias Rockbox.{Client, SavedPlaylist, SavedPlaylistFolder, Transport, Util}

  @playlist_fields "id name description image folderId trackCount createdAt updatedAt"
  @folder_fields "id name createdAt updatedAt"

  # ---------------------------------------------------------------------------
  # Playlists
  # ---------------------------------------------------------------------------

  @doc "List saved playlists, optionally filtered by `folder_id`."
  @spec list(Client.t(), String.t() | nil) :: {:ok, [SavedPlaylist.t()]} | {:error, Exception.t()}
  def list(client, folder_id \\ nil) do
    query =
      "query SavedPlaylists($folderId: String) { savedPlaylists(folderId: $folderId) { #{@playlist_fields} } }"

    with {:ok, %{"savedPlaylists" => list}} <-
           Transport.execute(client, query, %{folder_id: folder_id}) do
      {:ok, Util.to_struct_list(SavedPlaylist, list)}
    end
  end

  @spec list!(Client.t(), String.t() | nil) :: [SavedPlaylist.t()]
  def list!(client, folder_id \\ nil), do: bang(list(client, folder_id))

  @doc "Fetch a saved playlist by id."
  @spec get(Client.t(), String.t()) :: {:ok, SavedPlaylist.t() | nil} | {:error, Exception.t()}
  def get(client, id) do
    query = "query SavedPlaylist($id: String!) { savedPlaylist(id: $id) { #{@playlist_fields} } }"

    with {:ok, %{"savedPlaylist" => raw}} <- Transport.execute(client, query, %{id: id}) do
      {:ok, Util.to_struct(SavedPlaylist, raw)}
    end
  end

  @spec get!(Client.t(), String.t()) :: SavedPlaylist.t() | nil
  def get!(client, id), do: bang(get(client, id))

  @doc "Get the ordered track ids of a saved playlist."
  @spec track_ids(Client.t(), String.t()) :: {:ok, [String.t()]} | {:error, Exception.t()}
  def track_ids(client, playlist_id) do
    query =
      "query SavedPlaylistTrackIds($playlistId: String!) { savedPlaylistTrackIds(playlistId: $playlistId) }"

    with {:ok, %{"savedPlaylistTrackIds" => ids}} <-
           Transport.execute(client, query, %{playlist_id: playlist_id}),
         do: {:ok, ids}
  end

  @doc """
  Create a saved playlist.

  Required: `:name`. Optional: `:description`, `:image`, `:folder_id`, `:track_ids`.
  """
  @spec create(Client.t(), keyword() | map()) ::
          {:ok, SavedPlaylist.t()} | {:error, Exception.t()}
  def create(client, attrs) do
    vars = attrs |> Map.new() |> Map.take([:name, :description, :image, :folder_id, :track_ids])

    query =
      "mutation CreateSavedPlaylist($name: String!, $description: String, $image: String, $folderId: String, $trackIds: [String!]) { createSavedPlaylist(name: $name, description: $description, image: $image, folderId: $folderId, trackIds: $trackIds) { #{@playlist_fields} } }"

    with {:ok, %{"createSavedPlaylist" => raw}} <- Transport.execute(client, query, vars) do
      {:ok, Util.to_struct(SavedPlaylist, raw)}
    end
  end

  @spec create!(Client.t(), keyword() | map()) :: SavedPlaylist.t()
  def create!(client, attrs), do: bang(create(client, attrs))

  @doc "Update a saved playlist's metadata. Pass any subset of `:name`, `:description`, `:image`, `:folder_id`."
  @spec update(Client.t(), String.t(), keyword() | map()) :: :ok | {:error, Exception.t()}
  def update(client, id, attrs) do
    vars =
      attrs
      |> Map.new()
      |> Map.take([:name, :description, :image, :folder_id])
      |> Map.put(:id, id)

    void(
      Transport.execute(
        client,
        "mutation UpdateSavedPlaylist($id: String!, $name: String!, $description: String, $image: String, $folderId: String) { updateSavedPlaylist(id: $id, name: $name, description: $description, image: $image, folderId: $folderId) }",
        vars
      )
    )
  end

  @doc "Permanently delete a saved playlist."
  @spec delete(Client.t(), String.t()) :: :ok | {:error, Exception.t()}
  def delete(client, id),
    do:
      void(
        Transport.execute(
          client,
          "mutation DeleteSavedPlaylist($id: String!) { deleteSavedPlaylist(id: $id) }",
          %{id: id}
        )
      )

  @spec add_tracks(Client.t(), String.t(), [String.t()]) :: :ok | {:error, Exception.t()}
  def add_tracks(client, playlist_id, track_ids) do
    void(
      Transport.execute(
        client,
        "mutation AddTracksToSavedPlaylist($playlistId: String!, $trackIds: [String!]!) { addTracksToSavedPlaylist(playlistId: $playlistId, trackIds: $trackIds) }",
        %{playlist_id: playlist_id, track_ids: track_ids}
      )
    )
  end

  @spec remove_track(Client.t(), String.t(), String.t()) :: :ok | {:error, Exception.t()}
  def remove_track(client, playlist_id, track_id) do
    void(
      Transport.execute(
        client,
        "mutation RemoveTrackFromSavedPlaylist($playlistId: String!, $trackId: String!) { removeTrackFromSavedPlaylist(playlistId: $playlistId, trackId: $trackId) }",
        %{playlist_id: playlist_id, track_id: track_id}
      )
    )
  end

  @doc "Load this playlist into the active queue and start playing."
  @spec play(Client.t(), String.t()) :: :ok | {:error, Exception.t()}
  def play(client, playlist_id) do
    void(
      Transport.execute(
        client,
        "mutation PlaySavedPlaylist($playlistId: String!) { playSavedPlaylist(playlistId: $playlistId) }",
        %{playlist_id: playlist_id}
      )
    )
  end

  # ---------------------------------------------------------------------------
  # Folders
  # ---------------------------------------------------------------------------

  @spec folders(Client.t()) :: {:ok, [SavedPlaylistFolder.t()]} | {:error, Exception.t()}
  def folders(client) do
    query = "query PlaylistFolders { playlistFolders { #{@folder_fields} } }"

    with {:ok, %{"playlistFolders" => list}} <- Transport.execute(client, query) do
      {:ok, Util.to_struct_list(SavedPlaylistFolder, list)}
    end
  end

  @spec create_folder(Client.t(), String.t()) ::
          {:ok, SavedPlaylistFolder.t()} | {:error, Exception.t()}
  def create_folder(client, name) do
    with {:ok, %{"createPlaylistFolder" => raw}} <-
           Transport.execute(
             client,
             "mutation CreatePlaylistFolder($name: String!) { createPlaylistFolder(name: $name) { #{@folder_fields} } }",
             %{name: name}
           ) do
      {:ok, Util.to_struct(SavedPlaylistFolder, raw)}
    end
  end

  @spec delete_folder(Client.t(), String.t()) :: :ok | {:error, Exception.t()}
  def delete_folder(client, id),
    do:
      void(
        Transport.execute(
          client,
          "mutation DeletePlaylistFolder($id: String!) { deletePlaylistFolder(id: $id) }",
          %{id: id}
        )
      )

  defp void({:ok, _}), do: :ok
  defp void(err), do: err

  defp bang({:ok, value}), do: value
  defp bang(:ok), do: :ok
  defp bang({:error, exception}), do: raise(exception)
end
