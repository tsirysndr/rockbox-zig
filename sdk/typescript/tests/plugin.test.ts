import { describe, it, expect, vi } from 'vitest';
import { PluginRegistry } from '../src/plugin.js';
import type { RockboxPlugin, PluginContext } from '../src/plugin.js';

function makeContext(): PluginContext {
  return {
    query: vi.fn(),
    events: {
      on: vi.fn().mockReturnThis(),
      off: vi.fn().mockReturnThis(),
      once: vi.fn().mockReturnThis(),
      emit: vi.fn(),
      removeAllListeners: vi.fn().mockReturnThis(),
    } as unknown as PluginContext['events'],
  };
}

describe('PluginRegistry', () => {
  it('installs a plugin and calls install()', async () => {
    const registry = new PluginRegistry();
    const install = vi.fn();
    const plugin: RockboxPlugin = { name: 'scrobbler', version: '1.0.0', install };
    await registry.register(plugin, makeContext());
    expect(install).toHaveBeenCalledTimes(1);
    expect(registry.has('scrobbler')).toBe(true);
  });

  it('throws if the same plugin is installed twice', async () => {
    const registry = new PluginRegistry();
    const plugin: RockboxPlugin = { name: 'dupe', version: '1.0.0', install: vi.fn() };
    await registry.register(plugin, makeContext());
    await expect(registry.register(plugin, makeContext())).rejects.toThrow(/already installed/);
  });

  it('calls uninstall() when unregistering', async () => {
    const registry = new PluginRegistry();
    const uninstall = vi.fn();
    const plugin: RockboxPlugin = {
      name: 'lyrics',
      version: '1.0.0',
      install: vi.fn(),
      uninstall,
    };
    await registry.register(plugin, makeContext());
    await registry.unregister('lyrics');
    expect(uninstall).toHaveBeenCalledTimes(1);
    expect(registry.has('lyrics')).toBe(false);
  });

  it('list() returns all installed plugins', async () => {
    const registry = new PluginRegistry();
    const ctx = makeContext();
    await registry.register({ name: 'a', version: '1', install: vi.fn() }, ctx);
    await registry.register({ name: 'b', version: '1', install: vi.fn() }, ctx);
    expect(registry.list().map((p) => p.name)).toEqual(['a', 'b']);
  });

  it('unregistering a non-existent plugin is a no-op', async () => {
    const registry = new PluginRegistry();
    await expect(registry.unregister('ghost')).resolves.toBeUndefined();
  });
});
