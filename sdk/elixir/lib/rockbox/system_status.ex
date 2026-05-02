defmodule Rockbox.SystemStatus do
  @moduledoc "Aggregate runtime information about the rockboxd instance."

  @type t :: %__MODULE__{
          resume_index: integer(),
          resume_crc32: integer(),
          resume_elapsed: integer(),
          resume_offset: integer(),
          runtime: integer(),
          topruntime: integer(),
          dircache_size: integer(),
          last_screen: integer(),
          viewer_icon_count: integer(),
          last_volume_change: integer()
        }

  defstruct [
    :resume_index,
    :resume_crc32,
    :resume_elapsed,
    :resume_offset,
    :runtime,
    :topruntime,
    :dircache_size,
    :last_screen,
    :viewer_icon_count,
    :last_volume_change
  ]
end
