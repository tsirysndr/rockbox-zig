defmodule Rockbox.EqBand do
  @moduledoc "A single equalizer band setting."

  @type t :: %__MODULE__{
          cutoff: integer(),
          q: integer(),
          gain: integer()
        }

  defstruct [:cutoff, :q, :gain]
end
