# frozen_string_literal: true

require_relative "test_helper"

class CaseConversionTest < Minitest::Test
  CC = Rockbox::CaseConversion

  def test_camelize_simple
    assert_equal "albumId",     CC.camelize(:album_id)
    assert_equal "trackString", CC.camelize("track_string")
    assert_equal "id",          CC.camelize("id")
    assert_equal "",            CC.camelize("")
  end

  def test_camelize_leaves_single_word_lowercase
    assert_equal "title", CC.camelize("title")
  end

  def test_snakeize_simple
    assert_equal "album_id",     CC.snakeize("albumId")
    assert_equal "track_string", CC.snakeize("trackString")
    assert_equal "id",           CC.snakeize("id")
  end

  def test_snakeize_runs_of_caps
    # "trackHTTPUrl" -> the regex handles letter+CAP boundaries.
    assert_equal "track_http_url".tr("_", "_"), CC.snakeize("trackHttpUrl")
  end

  def test_deep_camelize_hash
    input  = { album_id: "abc", artist_name: "Pink Floyd" }
    output = CC.deep_camelize(input)
    assert_equal({ "albumId" => "abc", "artistName" => "Pink Floyd" }, output)
  end

  def test_deep_camelize_nested
    input = { settings: { player_name: "Living Room", eq_band_settings: [{ q_value: 1 }] } }
    out   = CC.deep_camelize(input)
    assert_equal "Living Room", out["settings"]["playerName"]
    assert_equal 1,             out["settings"]["eqBandSettings"][0]["qValue"]
  end

  def test_deep_camelize_passes_through_scalars
    assert_equal 42,    CC.deep_camelize(42)
    assert_equal "x",   CC.deep_camelize("x")
    assert_nil          CC.deep_camelize(nil)
    assert_equal true,  CC.deep_camelize(true)
  end

  def test_deep_snakeize_returns_symbol_keys
    input = { "albumId" => "abc", "trackString" => "1/12" }
    out   = CC.deep_snakeize(input)
    assert_equal "abc",  out[:album_id]
    assert_equal "1/12", out[:track_string]
  end

  def test_deep_snakeize_nested_with_array
    input = { "tracks" => [{ "trackId" => "t1" }, { "trackId" => "t2" }] }
    out   = CC.deep_snakeize(input)
    assert_equal "t1", out[:tracks][0][:track_id]
    assert_equal "t2", out[:tracks][1][:track_id]
  end

  def test_round_trip_preserves_structure
    original = { album_id: "x", tracks: [{ track_string: "1/12" }] }
    camelized = CC.deep_camelize(original)
    round_tripped = CC.deep_snakeize(camelized)
    assert_equal original, round_tripped
  end
end
