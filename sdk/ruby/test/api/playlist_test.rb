# frozen_string_literal: true

require_relative "../test_helper"

class PlaylistApiTest < Minitest::Test
  def setup
    @http     = Rockbox::FakeHttp.new
    @playlist = Rockbox::Api::Playlist.new(@http)
  end

  def test_current_builds_playlist_with_track_structs
    @http.will_return(playlist_get_current: {
      amount: 2, index: 0, max_playlist_size: 100, first_index: 0,
      last_insert_pos: 0, seed: 0, last_shuffled_start: 0,
      tracks: [
        { id: "t1", title: "Money" },
        { id: "t2", title: "Time" }
      ]
    })
    pl = @playlist.current
    assert_kind_of Rockbox::Playlist, pl
    assert_equal 2, pl.amount
    assert_equal 2, pl.tracks.size
    assert_kind_of Rockbox::Track, pl.tracks.first
    assert_equal "Money", pl.tracks.first.title
  end

  def test_current_when_response_empty
    @http.will_return(playlist_get_current: nil)
    pl = @playlist.current
    assert_kind_of Rockbox::Playlist, pl
    assert_equal [], pl.tracks
  end

  def test_amount_returns_integer
    @http.will_return(playlist_amount: 42)
    assert_equal 42, @playlist.amount
  end

  def test_insert_tracks_default_position_is_NEXT
    @playlist.insert_tracks(["/a.mp3", "/b.mp3"])
    assert_equal Rockbox::InsertPosition::NEXT, @http.last_call.variables[:position]
    assert_equal ["/a.mp3", "/b.mp3"],          @http.last_call.variables[:tracks]
    assert_nil   @http.last_call.variables[:playlist_id]
  end

  def test_insert_tracks_custom_position_and_playlist_id
    @playlist.insert_tracks(["/a.mp3"],
                            position: Rockbox::InsertPosition::LAST,
                            playlist_id: "pl_1")
    vars = @http.last_call.variables
    assert_equal Rockbox::InsertPosition::LAST, vars[:position]
    assert_equal "pl_1", vars[:playlist_id]
  end

  def test_insert_directory_default_is_LAST
    @playlist.insert_directory("/Music/Pink Floyd")
    assert_equal Rockbox::InsertPosition::LAST, @http.last_call.variables[:position]
    assert_equal "/Music/Pink Floyd",           @http.last_call.variables[:directory]
  end

  def test_insert_album_passes_album_id_and_position
    @playlist.insert_album("alb_1")
    assert_equal "alb_1", @http.last_call.variables[:album_id]
    assert_equal Rockbox::InsertPosition::LAST, @http.last_call.variables[:position]
  end

  def test_remove_track_passes_index
    @playlist.remove_track(3)
    assert_equal({ index: 3 }, @http.last_call.variables)
  end

  def test_clear_sends_no_variables
    @playlist.clear
    assert_nil @http.last_call.variables
  end

  def test_create_passes_name_and_tracks
    @playlist.create("Tonight", ["/a.mp3", "/b.mp3"])
    assert_equal({ name: "Tonight", tracks: ["/a.mp3", "/b.mp3"] }, @http.last_call.variables)
  end

  def test_start_compacts_nil_options
    @playlist.start
    assert_equal({}, @http.last_call.variables)
  end

  def test_start_includes_set_options_only
    @playlist.start(start_index: 0, elapsed: 1000)
    assert_equal({ start_index: 0, elapsed: 1000 }, @http.last_call.variables)
  end
end
