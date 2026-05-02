# frozen_string_literal: true

require_relative "test_helper"

class PluginTest < Minitest::Test
  class TestPlugin < Rockbox::Plugin
    attr_reader :installed_with, :uninstalled

    def initialize(name = "test-plugin")
      super()
      @plugin_name = name
      @uninstalled = false
    end

    def name;    @plugin_name; end
    def version; "1.0.0";      end

    def install(ctx)
      @installed_with = ctx
    end

    def uninstall
      @uninstalled = true
    end
  end

  def setup
    @registry = Rockbox::PluginRegistry.new
    @events   = Rockbox::EventEmitter.new
    @ctx      = Rockbox::PluginContext.new(query: ->(_, _ = nil) {}, events: @events)
  end

  def test_register_calls_install_with_context
    plugin = TestPlugin.new
    @registry.register(plugin, @ctx)
    assert_same @ctx, plugin.installed_with
    assert @registry.installed?("test-plugin")
  end

  def test_double_register_raises
    @registry.register(TestPlugin.new, @ctx)
    assert_raises(ArgumentError) { @registry.register(TestPlugin.new, @ctx) }
  end

  def test_unregister_calls_uninstall
    plugin = TestPlugin.new
    @registry.register(plugin, @ctx)
    @registry.unregister("test-plugin")
    assert plugin.uninstalled
    refute @registry.installed?("test-plugin")
  end

  def test_unregister_unknown_is_a_noop
    assert_nil @registry.unregister("nope")
  end

  def test_list_returns_registered_plugins
    a = TestPlugin.new("a")
    b = TestPlugin.new("b")
    @registry.register(a, @ctx)
    @registry.register(b, @ctx)
    assert_equal %w[a b], @registry.list.map(&:name).sort
  end

  def test_plugin_base_class_defaults
    base = Rockbox::Plugin.new
    assert_raises(NotImplementedError) { base.name }
    assert_equal "0.0.0", base.version
    assert_nil base.description
    assert_nil base.install(nil)
    assert_nil base.uninstall
  end

  def test_plugin_context_query_bang_delegates
    captured = nil
    ctx = Rockbox::PluginContext.new(
      query: ->(q, vars = nil) { captured = [q, vars]; { ok: true } },
      events: @events
    )
    result = ctx.query!("query Foo { foo }", { id: 1 })
    assert_equal({ ok: true }, result)
    assert_equal "query Foo { foo }", captured[0]
    assert_equal({ id: 1 }, captured[1])
  end
end
