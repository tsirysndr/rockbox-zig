import type { TypedEventEmitter, RockboxEventMap } from './events.js';

// ---------------------------------------------------------------------------
// Plugin system — inspired by Jellyfin's IPlugin / Kodi addon architecture
// ---------------------------------------------------------------------------

export interface PluginContext {
  /** HTTP + WebSocket transport — use to issue custom GraphQL operations */
  query<T>(gql: string, variables?: unknown): Promise<T>;
  /** Subscribe to typed SDK events */
  events: TypedEventEmitter<RockboxEventMap>;
}

export interface RockboxPlugin {
  /** Unique plugin identifier, e.g. "scrobbler" */
  readonly name: string;
  readonly version: string;
  readonly description?: string;
  install(context: PluginContext): void | Promise<void>;
  uninstall?(): void | Promise<void>;
}

export class PluginRegistry {
  private installed = new Map<string, RockboxPlugin>();

  async register(plugin: RockboxPlugin, context: PluginContext): Promise<void> {
    if (this.installed.has(plugin.name)) {
      throw new Error(`Plugin "${plugin.name}" is already installed`);
    }
    await plugin.install(context);
    this.installed.set(plugin.name, plugin);
  }

  async unregister(name: string): Promise<void> {
    const plugin = this.installed.get(name);
    if (!plugin) return;
    await plugin.uninstall?.();
    this.installed.delete(name);
  }

  has(name: string): boolean {
    return this.installed.has(name);
  }

  list(): RockboxPlugin[] {
    return [...this.installed.values()];
  }
}
