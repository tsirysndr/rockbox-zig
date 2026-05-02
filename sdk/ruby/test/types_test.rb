# frozen_string_literal: true

require_relative "test_helper"

class TypesTest < Minitest::Test
  def test_playback_status_constants
    assert_equal 0, Rockbox::PlaybackStatus::STOPPED
    assert_equal 1, Rockbox::PlaybackStatus::PLAYING
    assert_equal 3, Rockbox::PlaybackStatus::PAUSED
  end

  def test_playback_status_name_lookup
    assert_equal :stopped, Rockbox::PlaybackStatus.name(0)
    assert_equal :playing, Rockbox::PlaybackStatus.name(1)
    assert_equal :paused,  Rockbox::PlaybackStatus.name(3)
    assert_equal :unknown, Rockbox::PlaybackStatus.name(99)
    assert_equal :unknown, Rockbox::PlaybackStatus.name(nil)
  end

  def test_insert_position_constants
    assert_equal 0, Rockbox::InsertPosition::NEXT
    assert_equal 1, Rockbox::InsertPosition::AFTER_CURRENT
    assert_equal 2, Rockbox::InsertPosition::LAST
    assert_equal 3, Rockbox::InsertPosition::FIRST
  end

  def test_track_from_hash_extracts_known_fields
    hash = {
      id: "t1", title: "Money", artist: "Pink Floyd",
      length: 382_000, elapsed: 0,
      unknown_extension: "should be ignored"
    }
    track = Rockbox::Track.from_hash(hash)
    assert_equal "t1", track.id
    assert_equal "Money", track.title
    assert_equal 382_000, track.length
    refute track.respond_to?(:unknown_extension)
  end

  def test_track_from_hash_returns_nil_for_nil
    assert_nil Rockbox::Track.from_hash(nil)
  end

  def test_track_from_hash_missing_fields_default_to_nil
    track = Rockbox::Track.from_hash(id: "t1")
    assert_equal "t1", track.id
    assert_nil track.title
    assert_nil track.artist
  end

  def test_album_known_members
    assert_includes Rockbox::Album.known_members, :id
    assert_includes Rockbox::Album.known_members, :title
    assert_includes Rockbox::Album.known_members, :tracks
  end

  def test_volume_info_from_hash
    info = Rockbox::VolumeInfo.from_hash(volume: -10, min: -120, max: 12)
    assert_equal(-10, info.volume)
    assert_equal(-120, info.min)
    assert_equal 12, info.max
  end

  def test_directory_predicate_set_bit
    entry = Rockbox::Entry.new(name: "Pink Floyd", attr: 0x10, time_write: 0,
                               customaction: nil, display_name: nil)
    assert Rockbox.directory?(entry)
  end

  def test_directory_predicate_combined_bits
    # 0x18 has the dir bit set plus another flag.
    entry = Rockbox::Entry.new(name: "x", attr: 0x18, time_write: 0,
                               customaction: nil, display_name: nil)
    assert Rockbox.directory?(entry)
  end

  def test_directory_predicate_unset
    entry = Rockbox::Entry.new(name: "song.mp3", attr: 0, time_write: 0,
                               customaction: nil, display_name: nil)
    refute Rockbox.directory?(entry)
  end

  def test_directory_predicate_nil_attr
    entry = Rockbox::Entry.new(name: "x", attr: nil, time_write: nil,
                               customaction: nil, display_name: nil)
    refute Rockbox.directory?(entry)
  end

  def test_search_results_with_empty_arrays
    results = Rockbox::SearchResults.new(
      artists: [], albums: [], tracks: [], liked_tracks: [], liked_albums: []
    )
    assert_equal 0, results.tracks.size
  end
end
