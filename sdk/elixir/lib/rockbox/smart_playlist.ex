defmodule Rockbox.SmartPlaylist do
  @moduledoc """
  A rule-based playlist that resolves to a fresh track set every time it is
  played. Rules are stored as JSON; build them with
  `Rockbox.SmartPlaylist.Rules`.
  """

  @type t :: %__MODULE__{
          id: String.t(),
          name: String.t(),
          description: String.t() | nil,
          image: String.t() | nil,
          folder_id: String.t() | nil,
          is_system: boolean(),
          rules: String.t(),
          created_at: integer(),
          updated_at: integer()
        }

  defstruct [
    :id,
    :name,
    :description,
    :image,
    :folder_id,
    :is_system,
    :rules,
    :created_at,
    :updated_at
  ]
end
