defmodule Rockbox.SmartPlaylist.RulesTest do
  use ExUnit.Case, async: true

  alias Rockbox.SmartPlaylist.Rules

  test "all_of with where, sort, limit" do
    json =
      Rules.all_of()
      |> Rules.where(:play_count, :gte, 10)
      |> Rules.sort(:play_count, :desc)
      |> Rules.limit(50)
      |> Rules.to_json()

    decoded = Jason.decode!(json)
    assert decoded["operator"] == "AND"

    assert decoded["rules"] == [
             %{"field" => "play_count", "op" => "gte", "value" => 10}
           ]

    assert decoded["sort"] == %{"field" => "play_count", "dir" => "desc"}
    assert decoded["limit"] == 50
  end

  test "any_of starts an OR group" do
    map =
      Rules.any_of()
      |> Rules.where(:title, :contains, "Live")
      |> Rules.where(:title, :contains, "Acoustic")
      |> Rules.to_map()

    assert map.operator == "OR"
    assert length(map.rules) == 2
  end

  test "where_group nests sub-builders" do
    sub =
      Rules.any_of()
      |> Rules.where(:genre, :eq, "jazz")
      |> Rules.where(:genre, :eq, "blues")

    map =
      Rules.all_of()
      |> Rules.where(:play_count, :gt, 0)
      |> Rules.where_group(sub)
      |> Rules.to_map()

    assert length(map.rules) == 2
    assert Enum.at(map.rules, 1).operator == "OR"
  end
end
