# frozen_string_literal: true

require_relative "../test_helper"

class PlaybackApiTest < Minitest::Test
  def setup
    @http     = Rockbox::FakeHttp.new
    @playback = Rockbox::Api::Playback.new(@http)
  end

  def test_status_returns_integer
    @http.will_return(status: 1)
    assert_equal 1, @playback.status
  end

  def test_status_name_maps_through_playback_status
    @http.will_return(status: 3)
    assert_equal :paused, @playback.status_name
  end

  def test_current_track_builds_struct
    @http.will_return(current_track: { id: "t1", title: "Money", artist: "Pink Floyd" })
    track = @playback.current_track
    assert_kind_of Rockbox::Track, track
    assert_equal "Money", track.title
  end

  def test_current_track_returns_nil_when_absent
    @http.will_return(current_track: nil)
    assert_nil @playback.current_track
  end

  def test_play_sends_elapsed_and_offset
    @playback.play(elapsed: 1000, offset: 50)
    assert_equal({ elapsed: 1000, offset: 50 }, @http.last_call.variables)
  end

  def test_play_album_compacts_nil_options
    @playback.play_album("alb_1")
    assert_equal({ album_id: "alb_1" }, @http.last_call.variables)
  end

  def test_play_album_includes_explicit_options
    @playback.play_album("alb_1", shuffle: true, position: 0)
    assert_equal({ album_id: "alb_1", shuffle: true, position: 0 },
                 @http.last_call.variables)
  end

  def test_play_album_keeps_false_value
    @playback.play_album("alb_1", shuffle: false)
    assert_equal({ album_id: "alb_1", shuffle: false }, @http.last_call.variables)
  end

  def test_seek_uses_new_time_variable
    @playback.seek(60_000)
    assert_equal({ new_time: 60_000 }, @http.last_call.variables)
  end

  def test_simple_transport_controls_send_no_variables
    %i[pause resume next! previous! stop flush_and_reload].each do |method|
      @http.calls.clear
      @playback.public_send(method)
      assert_nil @http.last_call.variables, "#{method} should not send variables"
    end
  end
end
