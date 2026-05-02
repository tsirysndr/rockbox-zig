# Shared client factory used by every example.
#
# Override the host/port via env: ROCKBOX_HOST, ROCKBOX_PORT.

defmodule Examples.Helper do
  def client do
    Rockbox.new(
      host: System.get_env("ROCKBOX_HOST", "localhost"),
      port: String.to_integer(System.get_env("ROCKBOX_PORT", "6062"))
    )
  end

  @doc "Format milliseconds as M:SS."
  def fmt_time(ms), do: Rockbox.format_ms(ms)
end
