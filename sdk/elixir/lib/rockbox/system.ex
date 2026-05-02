defmodule Rockbox.System do
  @moduledoc "System-level information about the rockboxd instance."

  alias Rockbox.{Client, SystemStatus, Transport, Util}

  @doc "rockboxd version string."
  @spec version(Client.t()) :: {:ok, String.t()} | {:error, Exception.t()}
  def version(client) do
    case Transport.execute(client, "query Version { rockboxVersion }") do
      {:ok, %{"rockboxVersion" => v}} -> {:ok, v}
      err -> err
    end
  end

  @spec version!(Client.t()) :: String.t()
  def version!(client), do: bang(version(client))

  @doc "Aggregate runtime status."
  @spec status(Client.t()) :: {:ok, SystemStatus.t()} | {:error, Exception.t()}
  def status(client) do
    query = ~S"""
      query GlobalStatus {
        globalStatus {
          resumeIndex resumeCrc32 resumeElapsed resumeOffset
          runtime topruntime dircacheSize
          lastScreen viewerIconCount lastVolumeChange
        }
      }
    """

    with {:ok, %{"globalStatus" => raw}} <- Transport.execute(client, query) do
      {:ok, Util.to_struct(SystemStatus, raw)}
    end
  end

  @spec status!(Client.t()) :: SystemStatus.t()
  def status!(client), do: bang(status(client))

  defp bang({:ok, value}), do: value
  defp bang({:error, exception}), do: raise(exception)
end
