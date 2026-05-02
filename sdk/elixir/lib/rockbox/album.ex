defmodule Rockbox.Album do
  @moduledoc "An album in the music library. `:tracks` may be partially populated."

  @type t :: %__MODULE__{
          id: String.t(),
          title: String.t(),
          artist: String.t(),
          year: integer(),
          year_string: String.t(),
          album_art: String.t() | nil,
          md5: String.t(),
          artist_id: String.t(),
          copyright_message: String.t() | nil,
          tracks: [Rockbox.Track.t()]
        }

  defstruct [
    :id,
    :title,
    :artist,
    :year,
    :year_string,
    :album_art,
    :md5,
    :artist_id,
    :copyright_message,
    tracks: []
  ]
end
