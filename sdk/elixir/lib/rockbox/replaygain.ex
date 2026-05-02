defmodule Rockbox.Replaygain do
  @moduledoc "Replaygain settings."

  @type t :: %__MODULE__{
          noclip: boolean(),
          type: integer(),
          preamp: integer()
        }

  defstruct [:noclip, :type, :preamp]
end
