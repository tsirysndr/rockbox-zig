defmodule Rockbox.UserSettings do
  @moduledoc """
  Global user settings. Returned by `Rockbox.Settings.get/1` and accepted by
  `Rockbox.Settings.update/2` (any subset of fields can be supplied).
  """

  @type t :: %__MODULE__{
          music_dir: String.t(),
          volume: integer(),
          balance: integer(),
          bass: integer(),
          treble: integer(),
          channel_config: integer(),
          stereo_width: integer(),
          eq_enabled: boolean(),
          eq_precut: integer(),
          eq_band_settings: [Rockbox.EqBand.t()],
          replaygain_settings: Rockbox.Replaygain.t() | nil,
          compressor_settings: Rockbox.Compressor.t() | nil,
          crossfade_enabled: integer(),
          crossfade_fade_in_delay: integer(),
          crossfade_fade_in_duration: integer(),
          crossfade_fade_out_delay: integer(),
          crossfade_fade_out_duration: integer(),
          crossfade_fade_out_mixmode: integer(),
          crossfeed_enabled: boolean(),
          crossfeed_direct_gain: integer(),
          crossfeed_cross_gain: integer(),
          crossfeed_hf_attenuation: integer(),
          crossfeed_hf_cutoff: integer(),
          repeat_mode: integer(),
          single_mode: boolean(),
          party_mode: boolean(),
          shuffle: boolean(),
          player_name: String.t()
        }

  defstruct [
    :music_dir,
    :volume,
    :balance,
    :bass,
    :treble,
    :channel_config,
    :stereo_width,
    :eq_enabled,
    :eq_precut,
    :replaygain_settings,
    :compressor_settings,
    :crossfade_enabled,
    :crossfade_fade_in_delay,
    :crossfade_fade_in_duration,
    :crossfade_fade_out_delay,
    :crossfade_fade_out_duration,
    :crossfade_fade_out_mixmode,
    :crossfeed_enabled,
    :crossfeed_direct_gain,
    :crossfeed_cross_gain,
    :crossfeed_hf_attenuation,
    :crossfeed_hf_cutoff,
    :repeat_mode,
    :single_mode,
    :party_mode,
    :shuffle,
    :player_name,
    eq_band_settings: []
  ]
end
