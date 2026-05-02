# frozen_string_literal: true

module Rockbox
  # Lightweight typed event emitter. The set of valid events is documented
  # below; emitting an unknown event still works (no enforcement) so plugins
  # can publish their own.
  #
  # Built-in events:
  #
  # | Event              | Payload                              |
  # |--------------------|--------------------------------------|
  # | :track_changed     | Rockbox::Track                       |
  # | :status_changed    | Integer (Rockbox::PlaybackStatus)    |
  # | :playlist_changed  | Rockbox::Playlist                    |
  # | :ws_open           | nil                                  |
  # | :ws_close          | nil                                  |
  # | :ws_error          | Exception/StandardError              |
  class EventEmitter
    def initialize
      @listeners = Hash.new { |h, k| h[k] = [] }
      @lock = Mutex.new
    end

    # @example
    #   client.on(:track_changed) { |track| puts track.title }
    def on(event, &block)
      raise ArgumentError, "block required" unless block
      @lock.synchronize { @listeners[event.to_sym] << block }
      self
    end

    # @example
    #   client.once(:ws_open) { puts "connected!" }
    def once(event, &block)
      raise ArgumentError, "block required" unless block
      wrapper = nil
      wrapper = lambda do |*args|
        off(event, wrapper)
        block.call(*args)
      end
      on(event, &wrapper)
    end

    def off(event, listener = nil, &block)
      target = block || listener
      @lock.synchronize do
        if target.nil?
          @listeners.delete(event.to_sym)
        else
          @listeners[event.to_sym].delete(target)
        end
      end
      self
    end

    def emit(event, payload = nil)
      listeners = @lock.synchronize { @listeners[event.to_sym].dup }
      listeners.each do |listener|
        if listener.arity.zero? || payload.nil?
          listener.call
        else
          listener.call(payload)
        end
      end
    end

    def remove_all_listeners(event = nil)
      @lock.synchronize do
        if event
          @listeners.delete(event.to_sym)
        else
          @listeners.clear
        end
      end
      self
    end
  end
end
