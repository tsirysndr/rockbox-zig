# frozen_string_literal: true

module Rockbox
  # Base class for every error raised by the SDK.
  class Error < StandardError
    attr_reader :cause

    def initialize(message, cause: nil)
      super(message)
      @cause = cause
    end
  end

  # Raised when the HTTP/WebSocket transport cannot reach rockboxd.
  class NetworkError < Error; end

  # Raised when rockboxd returns a GraphQL `errors` payload.
  #
  # @example
  #   begin
  #     client.playback.play
  #   rescue Rockbox::GraphQLError => e
  #     puts e.errors.first[:message]
  #   end
  class GraphQLError < Error
    attr_reader :errors

    def initialize(errors)
      @errors = Array(errors)
      super(@errors.map { |e| e[:message] || e["message"] }.compact.join("; "))
    end
  end
end
