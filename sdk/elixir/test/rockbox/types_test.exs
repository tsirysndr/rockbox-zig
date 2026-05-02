defmodule Rockbox.TypesTest do
  use ExUnit.Case, async: true

  alias Rockbox.Types

  test "playback_status round-trips" do
    for atom <- [:stopped, :playing, :paused] do
      assert Types.playback_status(Types.from_playback_status(atom)) == atom
    end
  end

  test "repeat_mode round-trips" do
    for atom <- [:off, :all, :one, :shuffle, :ab_repeat] do
      assert Types.repeat_mode(Types.from_repeat_mode(atom)) == atom
    end
  end

  test "channel_config round-trips" do
    for atom <- [:stereo, :stereo_narrow, :mono, :left_mix, :right_mix, :karaoke] do
      assert Types.channel_config(Types.from_channel_config(atom)) == atom
    end
  end

  test "replaygain_type round-trips" do
    for atom <- [:track, :album, :shuffle] do
      assert Types.replaygain_type(Types.from_replaygain_type(atom)) == atom
    end
  end

  test "insert_position accepts atoms and ints" do
    assert Types.insert_position(:next) == 0
    assert Types.insert_position(:after_current) == 1
    assert Types.insert_position(:last) == 2
    assert Types.insert_position(:first) == 3
    assert Types.insert_position(7) == 7
  end

  test "unknown values surface as {:unknown, n}" do
    assert Types.playback_status(99) == {:unknown, 99}
  end
end
