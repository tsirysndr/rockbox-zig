# frozen_string_literal: true

require_relative "../test_helper"

class SettingsApiTest < Minitest::Test
  def setup
    @http     = Rockbox::FakeHttp.new
    @settings = Rockbox::Api::Settings.new(@http)
  end

  def test_get_builds_user_settings_struct
    @http.will_return(global_settings: {
      music_dir: "/Music", volume: -10, balance: 0, bass: 4, treble: 0,
      shuffle: true, player_name: "Living Room",
      eq_band_settings: [{ cutoff: 200, q: 1, gain: 3 }],
      replaygain_settings: { noclip: true, type: 1, preamp: 0 },
      compressor_settings: nil
    })
    s = @settings.get
    assert_kind_of Rockbox::UserSettings, s
    assert_equal "/Music",      s.music_dir
    assert_equal(-10,           s.volume)
    assert_equal "Living Room", s.player_name
    assert_equal 1,             s.eq_band_settings.size
    assert_kind_of Rockbox::EqBandSetting, s.eq_band_settings.first
    assert_equal 200,           s.eq_band_settings.first.cutoff
    assert_kind_of Rockbox::ReplaygainSettings, s.replaygain_settings
    assert_equal 1,             s.replaygain_settings.type
    assert_nil                  s.compressor_settings
  end

  def test_get_when_global_settings_missing
    @http.will_return(global_settings: nil)
    s = @settings.get
    assert_kind_of Rockbox::UserSettings, s
    assert_nil s.volume
    assert_equal [], s.eq_band_settings
  end

  def test_save_with_hash
    @settings.save(volume: -10, shuffle: true)
    assert_equal({ settings: { volume: -10, shuffle: true } },
                 @http.last_call.variables)
  end

  def test_save_with_block_uses_set_attrs
    @settings.save do |s|
      s.volume  = -10
      s.bass    = 4
      s.shuffle = true
    end
    assert_equal({ settings: { volume: -10, bass: 4, shuffle: true } },
                 @http.last_call.variables)
  end

  def test_save_block_attrs_can_be_extended_with_hash
    @settings.save({ player_name: "Kitchen" }) do |s|
      s.volume = -5
    end
    vars = @http.last_call.variables
    assert_equal(-5,        vars[:settings][:volume])
    assert_equal "Kitchen", vars[:settings][:player_name]
  end

  def test_save_with_no_args_raises
    assert_raises(ArgumentError) { @settings.save }
  end

  def test_save_with_empty_block_raises
    assert_raises(ArgumentError) { @settings.save {} }
  end
end
