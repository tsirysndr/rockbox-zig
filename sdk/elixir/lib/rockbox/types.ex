defmodule Rockbox.Types do
  @moduledoc """
  Enums and shared constants. Use the atom forms (`:playing`, `:stopped`, …)
  in your code; the SDK converts to the integer the server expects on the way
  in and back to atoms on the way out.

      iex> Rockbox.Types.playback_status(1)
      :playing

      iex> Rockbox.Types.from_playback_status(:paused)
      3

  ## Playback status

  | atom           | int |
  |----------------|-----|
  | `:stopped`     | 0   |
  | `:playing`     | 1   |
  | `:paused`      | 3   |

  ## Repeat mode

  | atom        | int |
  |-------------|-----|
  | `:off`      | 0   |
  | `:all`      | 1   |
  | `:one`      | 2   |
  | `:shuffle`  | 3   |
  | `:ab_repeat`| 4   |

  ## Channel config

  | atom            | int |
  |-----------------|-----|
  | `:stereo`       | 0   |
  | `:stereo_narrow`| 1   |
  | `:mono`         | 2   |
  | `:left_mix`     | 3   |
  | `:right_mix`    | 4   |
  | `:karaoke`      | 5   |

  ## Replaygain type

  | atom       | int |
  |------------|-----|
  | `:track`   | 0   |
  | `:album`   | 1   |
  | `:shuffle` | 2   |

  ## Insert position (queue)

  | atom            | int | Effect                                   |
  |-----------------|-----|------------------------------------------|
  | `:next`         | 0   | After the currently playing track        |
  | `:after_current`| 1   | After the last manually inserted track   |
  | `:last`         | 2   | At the end of the queue                  |
  | `:first`        | 3   | Replace the entire queue                 |
  """

  @type playback_status :: :stopped | :playing | :paused
  @type repeat_mode :: :off | :all | :one | :shuffle | :ab_repeat
  @type channel_config :: :stereo | :stereo_narrow | :mono | :left_mix | :right_mix | :karaoke
  @type replaygain_type :: :track | :album | :shuffle
  @type insert_position :: :next | :after_current | :last | :first

  # ---------------------------------------------------------------------------
  # Playback status
  # ---------------------------------------------------------------------------

  @spec playback_status(integer()) :: playback_status()
  def playback_status(0), do: :stopped
  def playback_status(1), do: :playing
  def playback_status(3), do: :paused
  def playback_status(other), do: {:unknown, other}

  @spec from_playback_status(playback_status()) :: integer()
  def from_playback_status(:stopped), do: 0
  def from_playback_status(:playing), do: 1
  def from_playback_status(:paused), do: 3

  # ---------------------------------------------------------------------------
  # Repeat mode
  # ---------------------------------------------------------------------------

  @spec repeat_mode(integer()) :: repeat_mode()
  def repeat_mode(0), do: :off
  def repeat_mode(1), do: :all
  def repeat_mode(2), do: :one
  def repeat_mode(3), do: :shuffle
  def repeat_mode(4), do: :ab_repeat
  def repeat_mode(other), do: {:unknown, other}

  @spec from_repeat_mode(repeat_mode()) :: integer()
  def from_repeat_mode(:off), do: 0
  def from_repeat_mode(:all), do: 1
  def from_repeat_mode(:one), do: 2
  def from_repeat_mode(:shuffle), do: 3
  def from_repeat_mode(:ab_repeat), do: 4

  # ---------------------------------------------------------------------------
  # Channel config
  # ---------------------------------------------------------------------------

  @spec channel_config(integer()) :: channel_config()
  def channel_config(0), do: :stereo
  def channel_config(1), do: :stereo_narrow
  def channel_config(2), do: :mono
  def channel_config(3), do: :left_mix
  def channel_config(4), do: :right_mix
  def channel_config(5), do: :karaoke
  def channel_config(other), do: {:unknown, other}

  @spec from_channel_config(channel_config()) :: integer()
  def from_channel_config(:stereo), do: 0
  def from_channel_config(:stereo_narrow), do: 1
  def from_channel_config(:mono), do: 2
  def from_channel_config(:left_mix), do: 3
  def from_channel_config(:right_mix), do: 4
  def from_channel_config(:karaoke), do: 5

  # ---------------------------------------------------------------------------
  # Replaygain
  # ---------------------------------------------------------------------------

  @spec replaygain_type(integer()) :: replaygain_type()
  def replaygain_type(0), do: :track
  def replaygain_type(1), do: :album
  def replaygain_type(2), do: :shuffle
  def replaygain_type(other), do: {:unknown, other}

  @spec from_replaygain_type(replaygain_type()) :: integer()
  def from_replaygain_type(:track), do: 0
  def from_replaygain_type(:album), do: 1
  def from_replaygain_type(:shuffle), do: 2

  # ---------------------------------------------------------------------------
  # Insert position
  # ---------------------------------------------------------------------------

  @spec insert_position(insert_position() | integer()) :: integer()
  def insert_position(:next), do: 0
  def insert_position(:after_current), do: 1
  def insert_position(:last), do: 2
  def insert_position(:first), do: 3
  def insert_position(int) when is_integer(int), do: int
end
