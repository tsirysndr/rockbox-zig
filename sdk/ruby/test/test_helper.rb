# frozen_string_literal: true

require "minitest/autorun"
require "json"

require "rockbox"

module Rockbox
  # Drop-in replacement for HttpTransport used in tests. Records every call
  # and returns scripted responses without touching the network.
  class FakeHttp
    Call = Struct.new(:query, :variables)

    attr_reader :calls

    def initialize
      @calls    = []
      @scripted = []
      @default  = {}
    end

    # Queue a response. The next #execute returns it (FIFO). If empty, the
    # default response is returned.
    def will_return(data)
      @scripted << data
      self
    end

    def default=(data)
      @default = data
    end

    # Match the HttpTransport contract.
    def execute(query, variables = nil)
      @calls << Call.new(query, variables)
      @scripted.empty? ? @default : @scripted.shift
    end

    # Convenience matchers for assertions.
    def last_call
      @calls.last
    end

    def call_count
      @calls.size
    end
  end
end
