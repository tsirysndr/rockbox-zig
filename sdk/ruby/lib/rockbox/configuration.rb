# frozen_string_literal: true

module Rockbox
  # Mutable configuration holder used by the builder block API.
  #
  # @example
  #   client = Rockbox::Client.build do |c|
  #     c.host = "192.168.1.42"
  #     c.port = 6062
  #   end
  class Configuration
    DEFAULT_HOST = "localhost"
    DEFAULT_PORT = 6062

    attr_accessor :host, :port, :http_url, :ws_url, :open_timeout, :read_timeout

    def initialize(host: nil, port: nil, http_url: nil, ws_url: nil,
                   open_timeout: nil, read_timeout: nil)
      @host         = host
      @port         = port
      @http_url     = http_url
      @ws_url       = ws_url
      @open_timeout = open_timeout
      @read_timeout = read_timeout
    end

    def resolved_host;  @host  || DEFAULT_HOST end
    def resolved_port;  @port  || DEFAULT_PORT end

    def resolved_http_url
      @http_url || "http://#{resolved_host}:#{resolved_port}/graphql"
    end

    def resolved_ws_url
      @ws_url || "ws://#{resolved_host}:#{resolved_port}/graphql"
    end
  end
end
