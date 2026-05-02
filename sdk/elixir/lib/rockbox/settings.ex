defmodule Rockbox.Settings do
  @moduledoc """
  Read and write global user settings. `update/2` accepts any subset of the
  fields in `Rockbox.UserSettings` — only the supplied keys are written.

      Rockbox.Settings.update(client,
        eq_enabled: true,
        eq_band_settings: [
          %{cutoff: 60,    q: 7, gain:  3},
          %{cutoff: 200,   q: 7, gain:  0},
          %{cutoff: 4000,  q: 7, gain: -2}
        ]
      )
  """

  alias Rockbox.{Client, Compressor, EqBand, Replaygain, Transport, UserSettings, Util}

  @fields ~S"""
    musicDir volume balance bass treble channelConfig stereoWidth
    eqEnabled eqPrecut
    eqBandSettings { cutoff q gain }
    replaygainSettings { noclip type preamp }
    compressorSettings { threshold makeupGain ratio knee releaseTime attackTime }
    crossfadeEnabled crossfadeFadeInDelay crossfadeFadeInDuration
    crossfadeFadeOutDelay crossfadeFadeOutDuration crossfadeFadeOutMixmode
    crossfeedEnabled crossfeedDirectGain crossfeedCrossGain
    crossfeedHfAttenuation crossfeedHfCutoff
    repeatMode singleMode partyMode shuffle playerName
  """

  @doc "Read the entire global settings object."
  @spec get(Client.t()) :: {:ok, UserSettings.t()} | {:error, Exception.t()}
  def get(client) do
    query = "query GlobalSettings { globalSettings { #{@fields} } }"

    with {:ok, %{"globalSettings" => raw}} <- Transport.execute(client, query) do
      atomized = Util.atomize(raw)
      base = Util.to_struct(UserSettings, raw)

      {:ok,
       %{
         base
         | eq_band_settings:
             Util.to_struct_list(EqBand, Map.get(atomized, :eq_band_settings, [])),
           replaygain_settings:
             Util.to_struct(Replaygain, Map.get(atomized, :replaygain_settings)),
           compressor_settings:
             Util.to_struct(Compressor, Map.get(atomized, :compressor_settings))
       }}
    end
  end

  @spec get!(Client.t()) :: UserSettings.t()
  def get!(client), do: bang(get(client))

  @doc """
  Save a partial settings update. Pass a keyword list or map of snake_case
  fields — they are converted to the camelCase shape the server expects.
  """
  @spec update(Client.t(), keyword() | map()) :: :ok | {:error, Exception.t()}
  def update(client, attrs) do
    settings = attrs |> Map.new() |> Util.camelize()

    void(
      Transport.execute(
        client,
        "mutation SaveSettings($settings: NewGlobalSettings!) { saveSettings(settings: $settings) }",
        %{"settings" => settings}
      )
    )
  end

  @spec update!(Client.t(), keyword() | map()) :: :ok
  def update!(client, attrs), do: bang(update(client, attrs))

  defp void({:ok, _}), do: :ok
  defp void(err), do: err

  defp bang({:ok, value}), do: value
  defp bang(:ok), do: :ok
  defp bang({:error, exception}), do: raise(exception)
end
