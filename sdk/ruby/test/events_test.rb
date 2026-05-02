# frozen_string_literal: true

require_relative "test_helper"

class EventsTest < Minitest::Test
  def setup
    @emitter = Rockbox::EventEmitter.new
  end

  def test_on_and_emit_passes_payload
    received = nil
    @emitter.on(:track_changed) { |t| received = t }
    @emitter.emit(:track_changed, "Money")
    assert_equal "Money", received
  end

  def test_on_returns_self_for_chaining
    assert_same @emitter, @emitter.on(:x) {}
  end

  def test_on_requires_block
    assert_raises(ArgumentError) { @emitter.on(:x) }
  end

  def test_emit_with_zero_arity_listener
    fired = 0
    @emitter.on(:ws_open) { fired += 1 }
    @emitter.emit(:ws_open)
    assert_equal 1, fired
  end

  def test_emit_no_payload_calls_listener_without_args
    fired = false
    @emitter.on(:ws_open) { |_payload| fired = true }
    @emitter.emit(:ws_open) # nil payload
    # listener arity is 1 but payload is nil — implementation calls without args.
    assert_equal true, fired
  end

  def test_string_event_name_normalized_to_symbol
    received = nil
    @emitter.on("track_changed") { |t| received = t }
    @emitter.emit("track_changed", "ok")
    assert_equal "ok", received
  end

  def test_multiple_listeners_fire_in_order
    log = []
    @emitter.on(:tick) { log << :a }
    @emitter.on(:tick) { log << :b }
    @emitter.emit(:tick)
    assert_equal %i[a b], log
  end

  def test_once_only_fires_once
    count = 0
    @emitter.once(:tick) { count += 1 }
    @emitter.emit(:tick)
    @emitter.emit(:tick)
    assert_equal 1, count
  end

  def test_off_with_proc_removes_specific_listener
    log = []
    listener = ->(p) { log << p }
    @emitter.on(:tick, &listener)
    @emitter.on(:tick) { log << :other }
    @emitter.off(:tick, listener)
    @emitter.emit(:tick, :payload)
    assert_equal [:other], log
  end

  def test_off_with_no_target_removes_all_listeners_for_event
    fired = 0
    @emitter.on(:tick) { fired += 1 }
    @emitter.on(:tick) { fired += 1 }
    @emitter.off(:tick)
    @emitter.emit(:tick)
    assert_equal 0, fired
  end

  def test_remove_all_listeners_clears_everything
    fired = 0
    @emitter.on(:a) { fired += 1 }
    @emitter.on(:b) { fired += 1 }
    @emitter.remove_all_listeners
    @emitter.emit(:a)
    @emitter.emit(:b)
    assert_equal 0, fired
  end

  def test_remove_all_listeners_for_one_event
    a, b = 0, 0
    @emitter.on(:a) { a += 1 }
    @emitter.on(:b) { b += 1 }
    @emitter.remove_all_listeners(:a)
    @emitter.emit(:a)
    @emitter.emit(:b)
    assert_equal 0, a
    assert_equal 1, b
  end
end
