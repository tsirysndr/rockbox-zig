defmodule Rockbox.Artist do
  @moduledoc "An artist with their albums and tracks."

  @type t :: %__MODULE__{
          id: String.t(),
          name: String.t(),
          bio: String.t() | nil,
          image: String.t() | nil,
          tracks: [Rockbox.Track.t()],
          albums: [Rockbox.Album.t()]
        }

  defstruct [:id, :name, :bio, :image, tracks: [], albums: []]
end
