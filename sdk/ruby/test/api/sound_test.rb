# frozen_string_literal: true

require_relative "../test_helper"

class SoundApiTest < Minitest::Test
  def setup
    @http  = Rockbox::FakeHttp.new
    @sound = Rockbox::Api::Sound.new(@http)
  end

  def test_volume_returns_struct
    @http.will_return(volume: { volume: -10, min: -120, max: 12 })
    info = @sound.volume
    assert_kind_of Rockbox::VolumeInfo, info
    assert_equal(-10, info.volume)
    assert_equal 12,  info.max
  end

  def test_adjust_sends_steps_and_returns_resulting_volume
    @http.will_return(adjust_volume: -5)
    result = @sound.adjust(3)
    assert_equal(-5, result)
    assert_equal({ steps: 3 }, @http.last_call.variables)
  end

  def test_up_sends_plus_one
    @http.will_return(adjust_volume: 0)
    @sound.up
    assert_equal({ steps: 1 }, @http.last_call.variables)
  end

  def test_down_sends_minus_one
    @http.will_return(adjust_volume: 0)
    @sound.down
    assert_equal({ steps: -1 }, @http.last_call.variables)
  end
end
