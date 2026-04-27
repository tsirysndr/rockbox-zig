import { HttpTransport, WsTransport } from './transport.js';
import { TypedEventEmitter, type RockboxEventMap } from './events.js';
import { PluginRegistry, type RockboxPlugin } from './plugin.js';

import { PlaybackApi } from './api/playback.js';
import { LibraryApi } from './api/library.js';
import { PlaylistApi } from './api/playlist.js';
import { SavedPlaylistsApi } from './api/saved-playlists.js';
import { SmartPlaylistsApi } from './api/smart-playlists.js';
import { SoundApi } from './api/sound.js';
import { SettingsApi } from './api/settings.js';
import { SystemApi } from './api/system.js';
import { BrowseApi } from './api/browse.js';
import { DevicesApi } from './api/devices.js';

import type { Track, Playlist } from './types.js';

export interface RockboxClientConfig {
  /** Hostname or IP of the rockboxd instance (default: "localhost") */
  host?: string;
  /** GraphQL HTTP/WS port (default: 6062) */
  port?: number;
  /** Override the full HTTP URL (takes precedence over host/port) */
  httpUrl?: string;
  /** Override the full WebSocket URL (takes precedence over host/port) */
  wsUrl?: string;
}

// ---------------------------------------------------------------------------
// RockboxClient — main entry point
// ---------------------------------------------------------------------------
// Inspired by:
//   Mopidy    — domain namespace API (client.playback.play(), client.library.search())
//   Jellyfin  — plugin system with install/uninstall lifecycle
//   Kodi      — rich device and playlist management
//   Navidrome — clean typed search & library queries
// ---------------------------------------------------------------------------

export class RockboxClient extends TypedEventEmitter<RockboxEventMap> {
  readonly playback: PlaybackApi;
  readonly library: LibraryApi;
  readonly playlist: PlaylistApi;
  readonly savedPlaylists: SavedPlaylistsApi;
  readonly smartPlaylists: SmartPlaylistsApi;
  readonly sound: SoundApi;
  readonly settings: SettingsApi;
  readonly system: SystemApi;
  readonly browse: BrowseApi;
  readonly devices: DevicesApi;

  private readonly http: HttpTransport;
  private readonly ws: WsTransport;
  private readonly plugins = new PluginRegistry();

  /** Unsubscribe handles returned by graphql-ws */
  private subscriptions: Array<() => void> = [];

  constructor(config: RockboxClientConfig = {}) {
    super();
    const host = config.host ?? 'localhost';
    const port = config.port ?? 6062;
    const httpUrl = config.httpUrl ?? `http://${host}:${port}/graphql`;
    const wsUrl = config.wsUrl ?? `ws://${host}:${port}/graphql`;

    this.http = new HttpTransport(httpUrl);
    this.ws = new WsTransport(wsUrl);

    this.playback = new PlaybackApi(this.http);
    this.library = new LibraryApi(this.http);
    this.playlist = new PlaylistApi(this.http);
    this.savedPlaylists = new SavedPlaylistsApi(this.http);
    this.smartPlaylists = new SmartPlaylistsApi(this.http);
    this.sound = new SoundApi(this.http);
    this.settings = new SettingsApi(this.http);
    this.system = new SystemApi(this.http);
    this.browse = new BrowseApi(this.http);
    this.devices = new DevicesApi(this.http);
  }

  // ---------------------------------------------------------------------------
  // Real-time subscriptions
  // ---------------------------------------------------------------------------

  /**
   * Start all GraphQL subscriptions and forward events to the event emitter.
   * Call once after construction. Safe to call multiple times (no-op if already started).
   */
  connect(): this {
    if (this.subscriptions.length > 0) return this;

    const trackSub = this.ws.subscribe<{ currentlyPlayingSong: Track }>(
      /* GraphQL */ `
        subscription CurrentlyPlaying {
          currentlyPlayingSong {
            id title artist album albumArt albumId artistId path length elapsed
          }
        }
      `,
      undefined,
      {
        next: ({ data }) => {
          if (data?.currentlyPlayingSong) {
            this.emit('track:changed', data.currentlyPlayingSong);
          }
        },
        error: (err) => this.emit('ws:error', err instanceof Error ? err : new Error(String(err))),
        complete: () => {},
      },
    );

    const statusSub = this.ws.subscribe<{ playbackStatus: { status: number } }>(
      /* GraphQL */ `subscription PlaybackStatus { playbackStatus { status } }`,
      undefined,
      {
        next: ({ data }) => {
          if (data?.playbackStatus != null) {
            this.emit('status:changed', data.playbackStatus.status);
          }
        },
        error: (err) => this.emit('ws:error', err instanceof Error ? err : new Error(String(err))),
        complete: () => {},
      },
    );

    const playlistSub = this.ws.subscribe<{ playlistChanged: Playlist }>(
      /* GraphQL */ `
        subscription PlaylistChanged {
          playlistChanged {
            amount index maxPlaylistSize firstIndex lastInsertPos seed lastShuffledStart
            tracks { id title artist album path length albumArt }
          }
        }
      `,
      undefined,
      {
        next: ({ data }) => {
          if (data?.playlistChanged) {
            this.emit('playlist:changed', data.playlistChanged);
          }
        },
        error: (err) => this.emit('ws:error', err instanceof Error ? err : new Error(String(err))),
        complete: () => {},
      },
    );

    this.subscriptions.push(trackSub, statusSub, playlistSub);
    return this;
  }

  /** Tear down all subscriptions and the WebSocket connection */
  disconnect(): void {
    for (const unsub of this.subscriptions) unsub();
    this.subscriptions = [];
    this.ws.dispose();
  }

  // ---------------------------------------------------------------------------
  // Plugin system (Jellyfin-style)
  // ---------------------------------------------------------------------------

  async use(plugin: RockboxPlugin): Promise<this> {
    await this.plugins.register(plugin, {
      query: (gql, variables) => this.http.execute(gql, variables),
      events: this,
    });
    return this;
  }

  async unuse(name: string): Promise<this> {
    await this.plugins.unregister(name);
    return this;
  }

  installedPlugins(): RockboxPlugin[] {
    return this.plugins.list();
  }

  // ---------------------------------------------------------------------------
  // Raw escape hatch — for consumers who need a one-off GraphQL call
  // ---------------------------------------------------------------------------

  query<T>(gql: string, variables?: unknown): Promise<T> {
    return this.http.execute<T>(gql, variables);
  }
}
