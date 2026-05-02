defmodule Rockbox.Device do
  @moduledoc "A discovered remote output device (Chromecast, AirPlay, …)."

  @type t :: %__MODULE__{
          id: String.t(),
          name: String.t(),
          host: String.t(),
          ip: String.t(),
          port: integer(),
          service: String.t(),
          app: String.t(),
          is_connected: boolean(),
          base_url: String.t() | nil,
          is_cast_device: boolean(),
          is_source_device: boolean(),
          is_current_device: boolean()
        }

  defstruct [
    :id,
    :name,
    :host,
    :ip,
    :port,
    :service,
    :app,
    :is_connected,
    :base_url,
    :is_cast_device,
    :is_source_device,
    :is_current_device
  ]
end
