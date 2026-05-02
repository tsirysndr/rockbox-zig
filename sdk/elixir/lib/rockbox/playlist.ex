defmodule Rockbox.Playlist do
  @moduledoc """
  A snapshot of the live playback queue. `:index` is the 0-based position of
  the currently playing track in `:tracks`.
  """

  @type t :: %__MODULE__{
          amount: integer(),
          index: integer(),
          max_playlist_size: integer(),
          first_index: integer(),
          last_insert_pos: integer(),
          seed: integer(),
          last_shuffled_start: integer(),
          tracks: [Rockbox.Track.t()]
        }

  defstruct [
    :amount,
    :index,
    :max_playlist_size,
    :first_index,
    :last_insert_pos,
    :seed,
    :last_shuffled_start,
    tracks: []
  ]

  @doc "The currently playing track, or `nil` if the queue is empty."
  @spec current_track(t()) :: Rockbox.Track.t() | nil
  def current_track(%__MODULE__{tracks: tracks, index: i}) when is_list(tracks) and i >= 0,
    do: Enum.at(tracks, i)

  def current_track(_), do: nil
end
