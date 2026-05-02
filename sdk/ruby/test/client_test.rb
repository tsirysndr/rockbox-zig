# frozen_string_literal: true

require_relative "test_helper"

# Tests Client wiring: builder DSL, top-level shorthand, event delegation,
# plugin lifecycle, and the raw #query escape hatch. We swap the HTTP
# transport for a FakeHttp on the constructed client to keep things offline.
class ClientTest < Minitest::Test
  def build_client
    client = Rockbox::Client.new(host: "localhost", port: 6062)
    fake = Rockbox::FakeHttp.new
    client.instance_variable_set(:@http, fake)
    [client, fake]
  end

  def test_default_namespaces_present
    client, = build_client
    %i[playback library playlist saved_playlists smart_playlists
       sound settings system browse devices bluetooth].each do |ns|
      refute_nil client.public_send(ns), "#{ns} should be present"
    end
  end

  def test_builder_dsl_yields_configuration
    client = Rockbox::Client.build do |c|
      c.host = "192.168.1.42"
      c.port = 7000
    end
    assert_equal "http://192.168.1.42:7000/graphql", client.configuration.resolved_http_url
  end

  def test_top_level_shorthand_with_kwargs
    client = Rockbox.new(host: "example.com")
    assert_kind_of Rockbox::Client, client
    assert_equal "example.com", client.configuration.host
  end

  def test_top_level_shorthand_with_block
    client = Rockbox.new do |c|
      c.host = "example.com"
    end
    assert_kind_of Rockbox::Client, client
    assert_equal "example.com", client.configuration.host
  end

  def test_event_delegation_to_emitter
    client, = build_client
    received = nil
    client.on(:track_changed) { |t| received = t }
    client.emit(:track_changed, "Money")
    assert_equal "Money", received
  end

  def test_once_via_client
    client, = build_client
    count = 0
    client.once(:tick) { count += 1 }
    client.emit(:tick)
    client.emit(:tick)
    assert_equal 1, count
  end

  def test_remove_all_listeners_via_client
    client, = build_client
    fired = false
    client.on(:tick) { fired = true }
    client.remove_all_listeners
    client.emit(:tick)
    refute fired
  end

  def test_raw_query_passes_through_http
    client, fake = build_client
    fake.will_return(some_field: 42)
    result = client.query("query Foo { foo }", { id: "abc" })
    assert_equal({ some_field: 42 }, result)
    assert_equal "query Foo { foo }", fake.last_call.query
    assert_equal({ id: "abc" }, fake.last_call.variables)
  end

  # ---- plugin lifecycle ---------------------------------------------------

  class CapturingPlugin < Rockbox::Plugin
    attr_reader :ctx

    def name;    "capture"; end
    def version; "0.0.1";   end

    def install(ctx)
      @ctx = ctx
    end
  end

  def test_use_installs_plugin_with_query_and_events
    client, fake = build_client
    plugin = CapturingPlugin.new
    client.use(plugin)
    assert_equal [plugin], client.installed_plugins

    # The query lambda routes through the same HTTP transport.
    fake.will_return(some_field: 1)
    plugin.ctx.query.call("query Foo { foo }")
    assert_equal "query Foo { foo }", fake.last_call.query

    # The events object lets plugins listen on the same emitter.
    received = nil
    plugin.ctx.events.on(:track_changed) { |t| received = t }
    client.emit(:track_changed, "Money")
    assert_equal "Money", received
  end

  def test_unuse_removes_plugin
    client, = build_client
    client.use(CapturingPlugin.new)
    client.unuse("capture")
    assert_empty client.installed_plugins
  end
end
