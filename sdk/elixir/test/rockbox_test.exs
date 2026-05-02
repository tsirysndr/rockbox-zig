defmodule RockboxTest do
  use ExUnit.Case, async: true
  doctest Rockbox

  describe "new/1" do
    test "defaults" do
      client = Rockbox.new()
      assert client.host == "localhost"
      assert client.port == 6062
      assert client.http_url == "http://localhost:6062/graphql"
      assert client.ws_url == "ws://localhost:6062/graphql"
    end

    test "host + port override" do
      client = Rockbox.new(host: "192.168.1.10", port: 7000)
      assert client.http_url == "http://192.168.1.10:7000/graphql"
      assert client.ws_url == "ws://192.168.1.10:7000/graphql"
    end

    test "http_url override takes precedence" do
      client = Rockbox.new(http_url: "https://music.home/api")
      assert client.http_url == "https://music.home/api"
    end
  end

  describe "format_ms/1" do
    test "formats sub-minute durations" do
      assert Rockbox.format_ms(45_000) == "0:45"
    end

    test "formats minute-and-seconds" do
      assert Rockbox.format_ms(75_000) == "1:15"
      assert Rockbox.format_ms(180_000) == "3:00"
    end

    test "pads single-digit seconds" do
      assert Rockbox.format_ms(61_000) == "1:01"
    end

    test "negative or invalid -> 0:00" do
      assert Rockbox.format_ms(-1) == "0:00"
      assert Rockbox.format_ms(nil) == "0:00"
    end
  end
end
