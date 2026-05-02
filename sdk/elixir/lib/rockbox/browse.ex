defmodule Rockbox.Browse do
  @moduledoc "Walk the filesystem relative to the configured `music_dir`."

  alias Rockbox.{Client, Entry, Transport, Util}

  @doc "List entries (files + directories) under `path`. `nil` = music_dir root."
  @spec entries(Client.t(), String.t() | nil) :: {:ok, [Entry.t()]} | {:error, Exception.t()}
  def entries(client, path \\ nil) do
    query = ~S"""
      query Browse($path: String) {
        treeGetEntries(path: $path) { name attr timeWrite customaction displayName }
      }
    """

    with {:ok, %{"treeGetEntries" => list}} <- Transport.execute(client, query, %{path: path}) do
      {:ok, Util.to_struct_list(Entry, list)}
    end
  end

  @spec entries!(Client.t(), String.t() | nil) :: [Entry.t()]
  def entries!(client, path \\ nil), do: bang(entries(client, path))

  @doc "Only the directories under `path`."
  @spec directories(Client.t(), String.t() | nil) :: {:ok, [Entry.t()]} | {:error, Exception.t()}
  def directories(client, path \\ nil) do
    with {:ok, entries} <- entries(client, path),
         do: {:ok, Enum.filter(entries, &Entry.directory?/1)}
  end

  @doc "Only the files under `path`."
  @spec files(Client.t(), String.t() | nil) :: {:ok, [Entry.t()]} | {:error, Exception.t()}
  def files(client, path \\ nil) do
    with {:ok, entries} <- entries(client, path),
         do: {:ok, Enum.filter(entries, &Entry.file?/1)}
  end

  defp bang({:ok, value}), do: value
  defp bang({:error, exception}), do: raise(exception)
end
