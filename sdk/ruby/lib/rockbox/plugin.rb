# frozen_string_literal: true

module Rockbox
  # Context handed to a plugin's #install method. Plugins can issue raw
  # GraphQL queries and subscribe to the same event stream the SDK uses.
  PluginContext = Struct.new(:query, :events, keyword_init: true) do
    def query!(*args, **kwargs)
      query.call(*args, **kwargs)
    end
  end

  # Plugin contract — duck-typed. A plugin must respond to:
  #   #name        => String  (unique)
  #   #version     => String
  #   #install(ctx) => any    (called when registered)
  #
  # Optional:
  #   #description => String
  #   #uninstall   => any     (called on unregister)
  #
  # Inherit from Plugin or just implement the methods directly.
  class Plugin
    def name; raise NotImplementedError; end
    def version; "0.0.0"; end
    def description; nil; end
    def install(_context); end
    def uninstall; end
  end

  class PluginRegistry
    def initialize
      @plugins = {}
      @lock = Mutex.new
    end

    def register(plugin, context)
      name = plugin.name.to_s
      @lock.synchronize do
        raise ArgumentError, "Plugin #{name.inspect} is already installed" if @plugins.key?(name)
      end
      plugin.install(context)
      @lock.synchronize { @plugins[name] = plugin }
      plugin
    end

    def unregister(name)
      plugin = @lock.synchronize { @plugins.delete(name.to_s) }
      plugin&.uninstall if plugin.respond_to?(:uninstall)
      plugin
    end

    def installed?(name)
      @lock.synchronize { @plugins.key?(name.to_s) }
    end

    def list
      @lock.synchronize { @plugins.values.dup }
    end
  end
end
