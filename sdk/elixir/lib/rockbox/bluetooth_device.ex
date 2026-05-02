defmodule Rockbox.BluetoothDevice do
  @moduledoc "A Bluetooth device known to the host (Linux only)."

  @type t :: %__MODULE__{
          address: String.t(),
          name: String.t(),
          paired: boolean(),
          trusted: boolean(),
          connected: boolean(),
          rssi: integer() | nil
        }

  defstruct [:address, :name, :paired, :trusted, :connected, :rssi]
end
