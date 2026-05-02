defmodule Rockbox.TrackStats do
  @moduledoc "Listening statistics for a track — feeds smart playlist rules."

  @type t :: %__MODULE__{
          track_id: String.t(),
          play_count: integer(),
          skip_count: integer(),
          last_played: integer() | nil,
          last_skipped: integer() | nil,
          updated_at: integer()
        }

  defstruct [:track_id, :play_count, :skip_count, :last_played, :last_skipped, :updated_at]
end
