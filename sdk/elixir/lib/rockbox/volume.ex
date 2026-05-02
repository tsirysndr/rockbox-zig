defmodule Rockbox.Volume do
  @moduledoc "Volume info: current value with the firmware-defined min/max range."

  @type t :: %__MODULE__{
          volume: integer(),
          min: integer(),
          max: integer()
        }

  defstruct [:volume, :min, :max]
end
