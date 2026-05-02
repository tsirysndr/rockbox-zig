# frozen_string_literal: true

require "json"
require "net/http"
require "uri"
require "securerandom"

require_relative "errors"
require_relative "case_conversion"

module Rockbox
  # ---------------------------------------------------------------------------
  # HTTP transport — POSTs GraphQL queries to rockboxd.
  #
  # Every outgoing variables hash is camelCased and every incoming `data`
  # payload is deep-snakeized so callers see idiomatic Ruby keys.
  # ---------------------------------------------------------------------------
  class HttpTransport
    DEFAULT_OPEN_TIMEOUT = 5
    DEFAULT_READ_TIMEOUT = 30

    def initialize(url, open_timeout: DEFAULT_OPEN_TIMEOUT, read_timeout: DEFAULT_READ_TIMEOUT)
      @uri = URI.parse(url)
      @open_timeout = open_timeout
      @read_timeout = read_timeout
    end

    # Execute a GraphQL operation. Returns the snake-cased `data` Hash.
    def execute(query, variables = nil)
      body = { query: query }
      body[:variables] = CaseConversion.deep_camelize(variables) if variables && !variables.empty?

      response = perform_request(body)
      parse_response(response)
    end

    private

    def perform_request(body)
      http = Net::HTTP.new(@uri.host, @uri.port)
      http.use_ssl = (@uri.scheme == "https")
      http.open_timeout = @open_timeout
      http.read_timeout = @read_timeout

      request = Net::HTTP::Post.new(@uri.request_uri)
      request["Content-Type"] = "application/json"
      request["Accept"]       = "application/json"
      request.body            = JSON.generate(body)

      http.request(request)
    rescue Errno::ECONNREFUSED, Errno::ETIMEDOUT, SocketError, Net::OpenTimeout, Net::ReadTimeout => e
      raise NetworkError.new("Failed to reach Rockbox at #{@uri}: #{e.message}", cause: e)
    end

    def parse_response(response)
      unless response.is_a?(Net::HTTPSuccess)
        raise NetworkError, "HTTP #{response.code} #{response.message}"
      end

      payload = JSON.parse(response.body)
      if (errors = payload["errors"]) && !errors.empty?
        raise GraphQLError, errors.map { |e| CaseConversion.deep_snakeize(e) }
      end

      CaseConversion.deep_snakeize(payload["data"]) || {}
    rescue JSON::ParserError => e
      raise NetworkError.new("Invalid JSON response: #{e.message}", cause: e)
    end
  end

  # ---------------------------------------------------------------------------
  # WebSocket transport — speaks the `graphql-transport-ws` protocol.
  #
  # Each call to {#subscribe} returns a "stop" lambda that cancels the
  # subscription. Reconnection is intentionally simple: the transport is
  # disposable, so callers should rebuild the client on terminal errors.
  # ---------------------------------------------------------------------------
  class WsTransport
    GRAPHQL_TRANSPORT_WS = "graphql-transport-ws"

    def initialize(url)
      @url = url
      @client = nil
      @lock = Mutex.new
      @sinks = {}        # subscription id => sink hash
      @ack = false
      @ack_signal = ConditionVariable.new
    end

    # @param query     [String]
    # @param variables [Hash, nil]
    # @param sink      [Hash{Symbol => Proc}] keys: :next, :error, :complete
    # @return          [Proc] call to unsubscribe
    def subscribe(query, variables, sink)
      ensure_connected

      sub_id = SecureRandom.uuid
      @lock.synchronize { @sinks[sub_id] = sink }

      send_message(
        id: sub_id,
        type: "subscribe",
        payload: {
          query: query,
          variables: variables ? CaseConversion.deep_camelize(variables) : {}
        }
      )

      lambda do
        send_message(id: sub_id, type: "complete") rescue nil
        @lock.synchronize { @sinks.delete(sub_id) }
      end
    end

    def dispose
      @lock.synchronize do
        @sinks.clear
        if @client
          begin
            @client.close
          rescue StandardError
            # ignored
          end
          @client = nil
          @ack = false
        end
      end
    end

    private

    def ensure_connected
      @lock.synchronize do
        return if @client && @ack

        require "websocket-client-simple" unless defined?(WebSocket::Client::Simple)

        transport = self
        @ack = false

        @client = WebSocket::Client::Simple.connect(@url, headers: { "Sec-WebSocket-Protocol" => GRAPHQL_TRANSPORT_WS })

        @client.on(:open)    { transport.send(:on_open) }
        @client.on(:message) { |msg| transport.send(:on_message, msg) }
        @client.on(:error)   { |err| transport.send(:on_error, err) }
        @client.on(:close)   { transport.send(:on_close) }

        # Wait for connection_ack before returning.
        deadline = Time.now + 5.0
        until @ack
          remaining = deadline - Time.now
          raise NetworkError, "Timed out waiting for graphql-transport-ws connection_ack" if remaining <= 0
          @ack_signal.wait(@lock, remaining)
        end
      end
    end

    def on_open
      send_message(type: "connection_init", payload: {})
    end

    def on_message(msg)
      data = msg.respond_to?(:data) ? msg.data : msg.to_s
      payload = JSON.parse(data)

      case payload["type"]
      when "connection_ack"
        @lock.synchronize do
          @ack = true
          @ack_signal.broadcast
        end
      when "next"
        sink = @lock.synchronize { @sinks[payload["id"]] }
        next_payload = payload["payload"] || {}
        data_hash = CaseConversion.deep_snakeize(next_payload["data"])
        sink&.dig(:next)&.call(data: data_hash)
      when "error"
        sink = @lock.synchronize { @sinks[payload["id"]] }
        sink&.dig(:error)&.call(payload["payload"])
      when "complete"
        sink = @lock.synchronize { @sinks.delete(payload["id"]) }
        sink&.dig(:complete)&.call
      end
    rescue JSON::ParserError
      # Drop malformed frames silently; rockboxd never sends them.
    end

    def on_error(err)
      sinks = @lock.synchronize { @sinks.values.dup }
      sinks.each { |s| s[:error]&.call(err) }
    end

    def on_close
      @lock.synchronize { @ack = false }
    end

    def send_message(message)
      raise NetworkError, "WebSocket is not connected" unless @client
      @client.send(JSON.generate(message))
    end
  end
end
