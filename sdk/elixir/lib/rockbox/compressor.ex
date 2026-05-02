defmodule Rockbox.Compressor do
  @moduledoc "Dynamics compressor settings."

  @type t :: %__MODULE__{
          threshold: integer(),
          makeup_gain: integer(),
          ratio: integer(),
          knee: integer(),
          release_time: integer(),
          attack_time: integer()
        }

  defstruct [:threshold, :makeup_gain, :ratio, :knee, :release_time, :attack_time]
end
