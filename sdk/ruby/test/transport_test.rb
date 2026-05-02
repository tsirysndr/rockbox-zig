# frozen_string_literal: true

require_relative "test_helper"

class HttpTransportTest < Minitest::Test
  # Stand-in for Net::HTTPSuccess.
  class FakeSuccess < Net::HTTPSuccess
    def initialize(body)
      super("1.1", "200", "OK")
      @fake_body = body
    end

    def body; @fake_body; end
  end

  class FakeFailure < Net::HTTPInternalServerError
    def initialize
      super("1.1", "500", "Internal Server Error")
    end

    def body; "boom"; end
  end

  # Patches Net::HTTP for the duration of a block: every request returns the
  # supplied response and we capture the request body for assertions.
  def with_stubbed_http(response, captured: [])
    Net::HTTP.class_eval do
      alias_method :__real_request, :request
      define_method(:request) do |req|
        captured << JSON.parse(req.body)
        response
      end
    end
    yield captured
  ensure
    Net::HTTP.class_eval do
      alias_method :request, :__real_request
      remove_method :__real_request
    end
  end

  def test_execute_returns_snake_cased_data
    captured = []
    response = FakeSuccess.new(JSON.generate("data" => { "albumId" => "abc" }))

    with_stubbed_http(response, captured: captured) do
      transport = Rockbox::HttpTransport.new("http://localhost:6062/graphql")
      result = transport.execute("query Foo { foo }")
      assert_equal({ album_id: "abc" }, result)
    end
  end

  def test_execute_camelizes_outgoing_variables
    captured = []
    response = FakeSuccess.new(JSON.generate("data" => {}))

    with_stubbed_http(response, captured: captured) do
      transport = Rockbox::HttpTransport.new("http://localhost:6062/graphql")
      transport.execute("mutation X($albumId: String!) { x(albumId: $albumId) }",
                        { album_id: "abc" })
    end

    body = captured.first
    assert_equal({ "albumId" => "abc" }, body["variables"])
  end

  def test_execute_omits_variables_when_empty
    captured = []
    response = FakeSuccess.new(JSON.generate("data" => {}))

    with_stubbed_http(response, captured: captured) do
      transport = Rockbox::HttpTransport.new("http://localhost:6062/graphql")
      transport.execute("query Foo { foo }")
      transport.execute("query Foo { foo }", {})
    end

    captured.each { |body| refute body.key?("variables") }
  end

  def test_execute_returns_empty_hash_when_data_is_nil
    response = FakeSuccess.new(JSON.generate("data" => nil))
    with_stubbed_http(response) do
      transport = Rockbox::HttpTransport.new("http://localhost:6062/graphql")
      assert_equal({}, transport.execute("query Foo { foo }"))
    end
  end

  def test_graphql_errors_raise_graphql_error
    response = FakeSuccess.new(JSON.generate(
      "data"   => nil,
      "errors" => [{ "message" => "missing argument" }]
    ))

    with_stubbed_http(response) do
      transport = Rockbox::HttpTransport.new("http://localhost:6062/graphql")
      err = assert_raises(Rockbox::GraphQLError) { transport.execute("query Bad { bad }") }
      assert_equal "missing argument", err.message
    end
  end

  def test_non_2xx_response_raises_network_error
    with_stubbed_http(FakeFailure.new) do
      transport = Rockbox::HttpTransport.new("http://localhost:6062/graphql")
      assert_raises(Rockbox::NetworkError) { transport.execute("query Foo { foo }") }
    end
  end

  def test_invalid_json_raises_network_error
    response = FakeSuccess.new("not json {{{")
    with_stubbed_http(response) do
      transport = Rockbox::HttpTransport.new("http://localhost:6062/graphql")
      assert_raises(Rockbox::NetworkError) { transport.execute("query Foo { foo }") }
    end
  end

  def test_econnrefused_raises_network_error
    Net::HTTP.class_eval do
      alias_method :__real_request, :request
      define_method(:request) { |_req| raise Errno::ECONNREFUSED }
    end

    transport = Rockbox::HttpTransport.new("http://localhost:6062/graphql")
    err = assert_raises(Rockbox::NetworkError) { transport.execute("query Foo { foo }") }
    assert_match(/Failed to reach Rockbox/, err.message)
    assert_kind_of Errno::ECONNREFUSED, err.cause
  ensure
    Net::HTTP.class_eval do
      alias_method :request, :__real_request
      remove_method :__real_request
    end
  end
end
