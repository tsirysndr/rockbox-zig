defmodule Rockbox.SmartPlaylist.Rules do
  @moduledoc """
  Composable builder for smart-playlist rule sets. Pipe-friendly, builder-friendly.

      alias Rockbox.SmartPlaylist.Rules

      rules =
        Rules.all_of()
        |> Rules.where(:play_count, :gte, 10)
        |> Rules.where(:last_played, :within, "30d")
        |> Rules.sort(:play_count, :desc)
        |> Rules.limit(50)
        |> Rules.to_json()

  The same builder works with `:any_of/0` for OR logic. The output is the
  JSON string the GraphQL `createSmartPlaylist` mutation expects.

  ## Common operators

  | atom        | meaning                              |
  |-------------|--------------------------------------|
  | `:eq`       | equals                               |
  | `:neq`      | not equals                           |
  | `:gt`       | greater than                         |
  | `:gte`      | greater than or equal                |
  | `:lt`       | less than                            |
  | `:lte`      | less than or equal                   |
  | `:contains` | substring match                      |
  | `:within`   | duration window (e.g. `"30d"`, `"7d"`) |
  """

  @type t :: %__MODULE__{
          operator: String.t(),
          rules: list(),
          sort: map() | nil,
          limit: pos_integer() | nil
        }

  defstruct operator: "AND", rules: [], sort: nil, limit: nil

  @doc "Start a builder where every rule must match (AND)."
  @spec all_of() :: t()
  def all_of, do: %__MODULE__{operator: "AND"}

  @doc "Start a builder where any single rule must match (OR)."
  @spec any_of() :: t()
  def any_of, do: %__MODULE__{operator: "OR"}

  @doc "Append a single rule. `op` is an atom; `value` is any JSON-encodable term."
  @spec where(t(), atom(), atom(), term()) :: t()
  def where(%__MODULE__{rules: rules} = builder, field, op, value) do
    %{builder | rules: rules ++ [%{field: to_string(field), op: to_string(op), value: value}]}
  end

  @doc "Nest a sub-builder. Useful for mixing AND/OR groups."
  @spec where_group(t(), t()) :: t()
  def where_group(%__MODULE__{rules: rules} = parent, %__MODULE__{} = child) do
    %{parent | rules: rules ++ [to_map(child)]}
  end

  @doc "Set the result ordering. `dir` is `:asc` or `:desc`."
  @spec sort(t(), atom(), :asc | :desc) :: t()
  def sort(%__MODULE__{} = builder, field, dir) when dir in [:asc, :desc] do
    %{builder | sort: %{field: to_string(field), dir: to_string(dir)}}
  end

  @doc "Cap the result count."
  @spec limit(t(), pos_integer()) :: t()
  def limit(%__MODULE__{} = builder, n) when is_integer(n) and n > 0 do
    %{builder | limit: n}
  end

  @doc "Render the builder as the plain map shape the server expects."
  @spec to_map(t()) :: map()
  def to_map(%__MODULE__{} = b) do
    base = %{operator: b.operator, rules: b.rules}
    base = if b.sort, do: Map.put(base, :sort, b.sort), else: base
    if b.limit, do: Map.put(base, :limit, b.limit), else: base
  end

  @doc "Render the builder as a JSON string ready for the smart-playlist mutation."
  @spec to_json(t()) :: String.t()
  def to_json(%__MODULE__{} = b), do: b |> to_map() |> Jason.encode!()
end
