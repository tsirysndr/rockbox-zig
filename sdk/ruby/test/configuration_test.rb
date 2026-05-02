# frozen_string_literal: true

require_relative "test_helper"

class ConfigurationTest < Minitest::Test
  def test_defaults
    config = Rockbox::Configuration.new
    assert_equal "localhost",                     config.resolved_host
    assert_equal 6062,                            config.resolved_port
    assert_equal "http://localhost:6062/graphql", config.resolved_http_url
    assert_equal "ws://localhost:6062/graphql",   config.resolved_ws_url
  end

  def test_kwargs_override_defaults
    config = Rockbox::Configuration.new(host: "192.168.1.42", port: 7000)
    assert_equal "http://192.168.1.42:7000/graphql", config.resolved_http_url
    assert_equal "ws://192.168.1.42:7000/graphql",   config.resolved_ws_url
  end

  def test_explicit_urls_take_precedence
    config = Rockbox::Configuration.new(
      host: "ignored",
      port: 1,
      http_url: "https://music.home/graphql",
      ws_url:   "wss://music.home/graphql"
    )
    assert_equal "https://music.home/graphql", config.resolved_http_url
    assert_equal "wss://music.home/graphql",   config.resolved_ws_url
  end

  def test_attr_writer_after_construction
    config = Rockbox::Configuration.new
    config.host = "example.com"
    config.port = 9000
    assert_equal "http://example.com:9000/graphql", config.resolved_http_url
  end

  def test_timeouts_passthrough
    config = Rockbox::Configuration.new(open_timeout: 1, read_timeout: 2)
    assert_equal 1, config.open_timeout
    assert_equal 2, config.read_timeout
  end
end
