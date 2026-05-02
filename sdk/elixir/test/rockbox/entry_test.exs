defmodule Rockbox.EntryTest do
  use ExUnit.Case, async: true

  alias Rockbox.Entry

  test "directory? checks the 0x10 bit" do
    assert Entry.directory?(%Entry{name: "x", attr: 0x10})
    refute Entry.directory?(%Entry{name: "x", attr: 0x00})
  end

  test "file? is the inverse" do
    assert Entry.file?(%Entry{name: "x", attr: 0x00})
    refute Entry.file?(%Entry{name: "x", attr: 0x10})
  end
end
