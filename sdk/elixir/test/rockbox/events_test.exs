defmodule Rockbox.EventsTest do
  use ExUnit.Case, async: false

  alias Rockbox.Events

  test "subscriber receives broadcast" do
    Events.subscribe(:track_changed)
    Events.broadcast(:track_changed, %Rockbox.Track{title: "Hello"})
    assert_receive {:rockbox, :track_changed, %Rockbox.Track{title: "Hello"}}, 200
  end

  test ":all subscriber gets every event" do
    Events.subscribe(:all)
    Events.broadcast(:status_changed, :playing)
    assert_receive {:rockbox, :status_changed, :playing}, 200
  end

  test "unsubscribe stops delivery" do
    Events.subscribe(:track_changed)
    Events.unsubscribe(:track_changed)
    Events.broadcast(:track_changed, %Rockbox.Track{title: "ignored"})
    refute_receive {:rockbox, :track_changed, _}, 100
  end

  test "subscribe/1 with a list" do
    Events.subscribe([:track_changed, :status_changed])
    Events.broadcast(:status_changed, :playing)
    Events.broadcast(:track_changed, %Rockbox.Track{title: "x"})
    assert_receive {:rockbox, :status_changed, :playing}
    assert_receive {:rockbox, :track_changed, %Rockbox.Track{}}
  end
end
