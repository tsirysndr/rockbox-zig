defmodule Rockbox.Track do
  @moduledoc "A single audio track. `:length` and `:elapsed` are milliseconds."

  @type t :: %__MODULE__{
          id: String.t() | nil,
          title: String.t(),
          artist: String.t(),
          album: String.t(),
          genre: String.t(),
          disc: String.t(),
          track_string: String.t(),
          year_string: String.t(),
          composer: String.t(),
          comment: String.t(),
          album_artist: String.t(),
          grouping: String.t(),
          discnum: integer(),
          tracknum: integer(),
          layer: integer(),
          year: integer(),
          bitrate: integer(),
          frequency: integer(),
          filesize: integer(),
          length: integer(),
          elapsed: integer(),
          path: String.t(),
          album_id: String.t() | nil,
          artist_id: String.t() | nil,
          genre_id: String.t() | nil,
          album_art: String.t() | nil
        }

  defstruct [
    :id,
    :title,
    :artist,
    :album,
    :genre,
    :disc,
    :track_string,
    :year_string,
    :composer,
    :comment,
    :album_artist,
    :grouping,
    :discnum,
    :tracknum,
    :layer,
    :year,
    :bitrate,
    :frequency,
    :filesize,
    :length,
    :elapsed,
    :path,
    :album_id,
    :artist_id,
    :genre_id,
    :album_art
  ]

  @doc "Format the track length as `M:SS`."
  @spec format_length(t()) :: String.t()
  def format_length(%__MODULE__{length: ms}) when is_integer(ms), do: Rockbox.format_ms(ms)
  def format_length(_), do: "0:00"

  @doc "Format the elapsed position as `M:SS`."
  @spec format_elapsed(t()) :: String.t()
  def format_elapsed(%__MODULE__{elapsed: ms}) when is_integer(ms), do: Rockbox.format_ms(ms)
  def format_elapsed(_), do: "0:00"

  @doc "Progress through the track as a 0.0–1.0 float."
  @spec progress(t()) :: float()
  def progress(%__MODULE__{length: len, elapsed: elapsed}) when is_integer(len) and len > 0,
    do: elapsed / len

  def progress(_), do: 0.0
end
