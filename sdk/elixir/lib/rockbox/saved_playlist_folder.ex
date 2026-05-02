defmodule Rockbox.SavedPlaylistFolder do
  @moduledoc "A folder grouping saved playlists together."

  @type t :: %__MODULE__{
          id: String.t(),
          name: String.t(),
          created_at: integer(),
          updated_at: integer()
        }

  defstruct [:id, :name, :created_at, :updated_at]
end
