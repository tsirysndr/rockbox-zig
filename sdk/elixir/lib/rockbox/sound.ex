defmodule Rockbox.Sound do
  @moduledoc """
  Volume controls. Volume is adjusted in firmware-defined steps — the absolute
  range varies by hardware target, so always check `volume/1` for `:min`/`:max`.
  """

  alias Rockbox.{Client, Transport, Util, Volume}

  @doc "Get the current volume with its hardware min/max range."
  @spec volume(Client.t()) :: {:ok, Volume.t()} | {:error, Exception.t()}
  def volume(client) do
    case Transport.execute(client, "query Volume { volume { volume min max } }") do
      {:ok, %{"volume" => raw}} -> {:ok, Util.to_struct(Volume, raw)}
      err -> err
    end
  end

  @spec volume!(Client.t()) :: Volume.t()
  def volume!(client), do: bang(volume(client))

  @doc "Adjust volume by a relative number of steps (positive = louder)."
  @spec adjust(Client.t(), integer()) :: {:ok, integer()} | {:error, Exception.t()}
  def adjust(client, steps) when is_integer(steps) do
    case Transport.execute(
           client,
           "mutation AdjustVolume($steps: Int!) { adjustVolume(steps: $steps) }",
           %{steps: steps}
         ) do
      {:ok, %{"adjustVolume" => raw}} -> {:ok, raw}
      err -> err
    end
  end

  @spec up(Client.t()) :: {:ok, integer()} | {:error, Exception.t()}
  def up(client), do: adjust(client, 1)

  @spec down(Client.t()) :: {:ok, integer()} | {:error, Exception.t()}
  def down(client), do: adjust(client, -1)

  defp bang({:ok, value}), do: value
  defp bang({:error, exception}), do: raise(exception)
end
