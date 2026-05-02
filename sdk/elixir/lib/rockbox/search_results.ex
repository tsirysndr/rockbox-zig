defmodule Rockbox.SearchResults do
  @moduledoc "Aggregated results returned by `Rockbox.Library.search/2`."

  @type t :: %__MODULE__{
          artists: [Rockbox.Artist.t()],
          albums: [Rockbox.Album.t()],
          tracks: [Rockbox.Track.t()],
          liked_tracks: [Rockbox.Track.t()],
          liked_albums: [Rockbox.Album.t()]
        }

  defstruct artists: [], albums: [], tracks: [], liked_tracks: [], liked_albums: []
end
