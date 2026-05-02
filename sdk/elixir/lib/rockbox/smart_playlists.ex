defmodule Rockbox.SmartPlaylists do
  @moduledoc """
  Rule-based playlists that resolve to a fresh track set every time they are
  played.

  Build the rule JSON manually with `Jason.encode!/1`, or use
  `Rockbox.SmartPlaylist.Rules` for a typed builder:

      rules =
        Rockbox.SmartPlaylist.Rules.all_of()
        |> Rockbox.SmartPlaylist.Rules.where(:play_count, :gte, 10)
        |> Rockbox.SmartPlaylist.Rules.sort(:play_count, :desc)
        |> Rockbox.SmartPlaylist.Rules.limit(50)
        |> Rockbox.SmartPlaylist.Rules.to_json()

      {:ok, sp} = Rockbox.SmartPlaylists.create(client, name: "Top 50", rules: rules)
  """

  alias Rockbox.{Client, SmartPlaylist, TrackStats, Transport, Util}

  @playlist_fields "id name description image folderId isSystem rules createdAt updatedAt"
  @stats_fields "trackId playCount skipCount lastPlayed lastSkipped updatedAt"

  @spec list(Client.t()) :: {:ok, [SmartPlaylist.t()]} | {:error, Exception.t()}
  def list(client) do
    query = "query SmartPlaylists { smartPlaylists { #{@playlist_fields} } }"

    with {:ok, %{"smartPlaylists" => list}} <- Transport.execute(client, query) do
      {:ok, Util.to_struct_list(SmartPlaylist, list)}
    end
  end

  @spec list!(Client.t()) :: [SmartPlaylist.t()]
  def list!(client), do: bang(list(client))

  @spec get(Client.t(), String.t()) :: {:ok, SmartPlaylist.t() | nil} | {:error, Exception.t()}
  def get(client, id) do
    query = "query SmartPlaylist($id: String!) { smartPlaylist(id: $id) { #{@playlist_fields} } }"

    with {:ok, %{"smartPlaylist" => raw}} <- Transport.execute(client, query, %{id: id}) do
      {:ok, Util.to_struct(SmartPlaylist, raw)}
    end
  end

  @doc "Resolve the rule set right now and return matching track ids."
  @spec track_ids(Client.t(), String.t()) :: {:ok, [String.t()]} | {:error, Exception.t()}
  def track_ids(client, id) do
    with {:ok, %{"smartPlaylistTrackIds" => ids}} <-
           Transport.execute(
             client,
             "query SmartPlaylistTrackIds($id: String!) { smartPlaylistTrackIds(id: $id) }",
             %{id: id}
           ),
         do: {:ok, ids}
  end

  @doc """
  Create a smart playlist.

  Required: `:name`, `:rules` (JSON string).
  Optional: `:description`, `:image`, `:folder_id`.
  """
  @spec create(Client.t(), keyword() | map()) ::
          {:ok, SmartPlaylist.t()} | {:error, Exception.t()}
  def create(client, attrs) do
    vars =
      attrs
      |> Map.new()
      |> Map.take([:name, :rules, :description, :image, :folder_id])

    query =
      "mutation CreateSmartPlaylist($name: String!, $rules: String!, $description: String, $image: String, $folderId: String) { createSmartPlaylist(name: $name, rules: $rules, description: $description, image: $image, folderId: $folderId) { #{@playlist_fields} } }"

    with {:ok, %{"createSmartPlaylist" => raw}} <- Transport.execute(client, query, vars) do
      {:ok, Util.to_struct(SmartPlaylist, raw)}
    end
  end

  @spec create!(Client.t(), keyword() | map()) :: SmartPlaylist.t()
  def create!(client, attrs), do: bang(create(client, attrs))

  @doc "Update a smart playlist. Required: `:name`, `:rules`."
  @spec update(Client.t(), String.t(), keyword() | map()) :: :ok | {:error, Exception.t()}
  def update(client, id, attrs) do
    vars =
      attrs
      |> Map.new()
      |> Map.take([:name, :rules, :description, :image, :folder_id])
      |> Map.put(:id, id)

    void(
      Transport.execute(
        client,
        "mutation UpdateSmartPlaylist($id: String!, $name: String!, $rules: String!, $description: String, $image: String, $folderId: String) { updateSmartPlaylist(id: $id, name: $name, rules: $rules, description: $description, image: $image, folderId: $folderId) }",
        vars
      )
    )
  end

  @spec delete(Client.t(), String.t()) :: :ok | {:error, Exception.t()}
  def delete(client, id),
    do:
      void(
        Transport.execute(
          client,
          "mutation DeleteSmartPlaylist($id: String!) { deleteSmartPlaylist(id: $id) }",
          %{id: id}
        )
      )

  @spec play(Client.t(), String.t()) :: :ok | {:error, Exception.t()}
  def play(client, id),
    do:
      void(
        Transport.execute(
          client,
          "mutation PlaySmartPlaylist($id: String!) { playSmartPlaylist(id: $id) }",
          %{id: id}
        )
      )

  # ---------------------------------------------------------------------------
  # Listening stats
  # ---------------------------------------------------------------------------

  @spec track_stats(Client.t(), String.t()) ::
          {:ok, TrackStats.t() | nil} | {:error, Exception.t()}
  def track_stats(client, track_id) do
    query =
      "query TrackStats($trackId: String!) { trackStats(trackId: $trackId) { #{@stats_fields} } }"

    with {:ok, %{"trackStats" => raw}} <- Transport.execute(client, query, %{track_id: track_id}) do
      {:ok, Util.to_struct(TrackStats, raw)}
    end
  end

  @spec record_played(Client.t(), String.t()) :: :ok | {:error, Exception.t()}
  def record_played(client, track_id),
    do:
      void(
        Transport.execute(
          client,
          "mutation RecordTrackPlayed($trackId: String!) { recordTrackPlayed(trackId: $trackId) }",
          %{track_id: track_id}
        )
      )

  @spec record_skipped(Client.t(), String.t()) :: :ok | {:error, Exception.t()}
  def record_skipped(client, track_id),
    do:
      void(
        Transport.execute(
          client,
          "mutation RecordTrackSkipped($trackId: String!) { recordTrackSkipped(trackId: $trackId) }",
          %{track_id: track_id}
        )
      )

  defp void({:ok, _}), do: :ok
  defp void(err), do: err

  defp bang({:ok, value}), do: value
  defp bang(:ok), do: :ok
  defp bang({:error, exception}), do: raise(exception)
end
