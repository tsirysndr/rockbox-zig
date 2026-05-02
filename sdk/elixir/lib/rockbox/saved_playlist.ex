defmodule Rockbox.SavedPlaylist do
  @moduledoc "A persistent named playlist stored in the library database."

  @type t :: %__MODULE__{
          id: String.t(),
          name: String.t(),
          description: String.t() | nil,
          image: String.t() | nil,
          folder_id: String.t() | nil,
          track_count: integer(),
          created_at: integer(),
          updated_at: integer()
        }

  defstruct [
    :id,
    :name,
    :description,
    :image,
    :folder_id,
    :track_count,
    :created_at,
    :updated_at
  ]
end
