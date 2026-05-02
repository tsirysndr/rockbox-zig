defmodule Rockbox.Util do
  @moduledoc false

  # Internal helpers — JSON key conversion between the GraphQL camelCase wire
  # format and idiomatic snake_case Elixir keys, plus typed enum coercion.

  @doc "Recursively convert string camelCase map keys to snake_case atoms."
  @spec atomize(term()) :: term()
  def atomize(value) when is_list(value), do: Enum.map(value, &atomize/1)

  def atomize(%{} = map) do
    Map.new(map, fn {k, v} -> {to_snake_atom(k), atomize(v)} end)
  end

  def atomize(other), do: other

  @doc "Recursively convert snake_case atom map keys to camelCase strings."
  @spec camelize(term()) :: term()
  def camelize(value) when is_list(value), do: Enum.map(value, &camelize/1)

  def camelize(%{} = map) do
    Map.new(map, fn {k, v} -> {to_camel_string(k), camelize(v)} end)
  end

  def camelize(other), do: other

  @doc "Build a struct from a (possibly camelCase) map, ignoring unknown keys."
  @spec to_struct(module(), map() | nil) :: struct() | nil
  def to_struct(_module, nil), do: nil

  def to_struct(module, %{} = map) do
    atomized = atomize(map)
    blank = struct(module)
    fields = blank |> Map.from_struct() |> Map.keys()
    relevant = Map.take(atomized, fields)
    struct(module, relevant)
  end

  @doc "Build a list of structs from a list of maps."
  @spec to_struct_list(module(), [map()] | nil) :: [struct()]
  def to_struct_list(_module, nil), do: []
  def to_struct_list(module, list), do: Enum.map(list, &to_struct(module, &1))

  @doc "Convert a keyword list of options into a camelCase variables map for GraphQL."
  @spec opts_to_variables(Keyword.t() | map()) :: map()
  def opts_to_variables(opts) when is_list(opts), do: opts |> Map.new() |> camelize()
  def opts_to_variables(opts) when is_map(opts), do: camelize(opts)

  # ---------------------------------------------------------------------------
  # Internal
  # ---------------------------------------------------------------------------

  defp to_snake_atom(key) when is_atom(key), do: key

  defp to_snake_atom(key) when is_binary(key) do
    key
    |> String.replace(~r/([a-z0-9])([A-Z])/, "\\1_\\2")
    |> String.downcase()
    |> String.to_atom()
  end

  defp to_camel_string(key) when is_binary(key), do: to_camel_string(String.to_atom(key))

  defp to_camel_string(key) when is_atom(key) do
    [head | rest] = key |> Atom.to_string() |> String.split("_")
    head <> Enum.map_join(rest, "", &String.capitalize/1)
  end
end
