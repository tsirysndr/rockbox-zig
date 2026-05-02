# frozen_string_literal: true

require_relative "../test_helper"

class BrowseApiTest < Minitest::Test
  def setup
    @http   = Rockbox::FakeHttp.new
    @browse = Rockbox::Api::Browse.new(@http)
  end

  def test_entries_returns_struct_array
    @http.will_return(tree_get_entries: [
      { name: "Pink Floyd", attr: 0x10, time_write: 0, customaction: nil, display_name: "Pink Floyd" },
      { name: "song.mp3",   attr: 0x00, time_write: 0, customaction: nil, display_name: "song.mp3" }
    ])
    entries = @browse.entries("/Music")
    assert_equal 2, entries.size
    assert_kind_of Rockbox::Entry, entries.first
  end

  def test_entries_with_nil_path_sends_no_variables
    @http.will_return(tree_get_entries: [])
    @browse.entries
    assert_nil @http.last_call.variables
  end

  def test_entries_with_path_sends_path_variable
    @http.will_return(tree_get_entries: [])
    @browse.entries("/Music")
    assert_equal({ path: "/Music" }, @http.last_call.variables)
  end

  def test_directories_filters_dir_attr_only
    @http.will_return(tree_get_entries: [
      { name: "Pink Floyd", attr: 0x10, time_write: 0, customaction: nil, display_name: nil },
      { name: "song.mp3",   attr: 0x00, time_write: 0, customaction: nil, display_name: nil }
    ])
    dirs = @browse.directories
    assert_equal 1, dirs.size
    assert_equal "Pink Floyd", dirs.first.name
  end

  def test_files_excludes_directories
    @http.will_return(tree_get_entries: [
      { name: "Pink Floyd", attr: 0x10, time_write: 0, customaction: nil, display_name: nil },
      { name: "song.mp3",   attr: 0x00, time_write: 0, customaction: nil, display_name: nil }
    ])
    files = @browse.files
    assert_equal 1, files.size
    assert_equal "song.mp3", files.first.name
  end
end
