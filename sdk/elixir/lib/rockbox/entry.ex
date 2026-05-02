defmodule Rockbox.Entry do
  @moduledoc """
  A filesystem entry returned by `Rockbox.Browse`. `:attr` is a bitmask;
  use `directory?/1` instead of inspecting it directly.
  """

  @type t :: %__MODULE__{
          name: String.t(),
          attr: integer(),
          time_write: integer(),
          customaction: integer(),
          display_name: String.t() | nil
        }

  defstruct [:name, :attr, :time_write, :customaction, :display_name]

  @doc "Returns `true` when the entry is a directory (attr bit 4 set)."
  @spec directory?(t()) :: boolean()
  def directory?(%__MODULE__{attr: attr}) when is_integer(attr), do: Bitwise.band(attr, 0x10) != 0
  def directory?(_), do: false

  @doc "Returns `true` when the entry is a regular file."
  @spec file?(t()) :: boolean()
  def file?(entry), do: not directory?(entry)
end
