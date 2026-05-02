defmodule Rockbox.UtilTest do
  use ExUnit.Case, async: true

  alias Rockbox.Util

  describe "atomize/1" do
    test "converts camelCase string keys to snake_case atoms" do
      assert Util.atomize(%{"trackNum" => 1, "albumArt" => "x"}) == %{
               track_num: 1,
               album_art: "x"
             }
    end

    test "recurses into nested maps" do
      input = %{"outer" => %{"innerKey" => 1}}
      assert Util.atomize(input) == %{outer: %{inner_key: 1}}
    end

    test "recurses into lists of maps" do
      input = [%{"firstName" => "a"}, %{"firstName" => "b"}]
      assert Util.atomize(input) == [%{first_name: "a"}, %{first_name: "b"}]
    end

    test "leaves non-map/list values alone" do
      assert Util.atomize("hi") == "hi"
      assert Util.atomize(42) == 42
      assert Util.atomize(nil) == nil
    end
  end

  describe "camelize/1" do
    test "converts snake_case atom keys to camelCase strings" do
      assert Util.camelize(%{album_id: "x", play_count: 5}) ==
               %{"albumId" => "x", "playCount" => 5}
    end

    test "passes through values" do
      assert Util.camelize(%{name: "foo"}) == %{"name" => "foo"}
    end

    test "round-trips through atomize/1" do
      original = %{"trackNum" => 1, "nested" => %{"theKey" => 42}}
      assert original |> Util.atomize() |> Util.camelize() == original
    end
  end

  describe "to_struct/2" do
    test "builds a struct, ignoring unknown keys" do
      raw = %{"id" => "x", "title" => "Hi", "extraField" => "ignored"}
      track = Util.to_struct(Rockbox.Track, raw)
      assert track.id == "x"
      assert track.title == "Hi"
    end

    test "returns nil for nil input" do
      assert Util.to_struct(Rockbox.Track, nil) == nil
    end
  end
end
