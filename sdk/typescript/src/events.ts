import type { Track, Playlist } from './types.js';

// ---------------------------------------------------------------------------
// Typed event map — all events the SDK emits
// ---------------------------------------------------------------------------

export interface RockboxEventMap {
  /** Fires whenever the currently playing track changes */
  'track:changed': Track;
  /** Fires whenever the playback status changes (0=stopped, 1=playing, 2=paused) */
  'status:changed': number;
  /** Fires whenever the current playlist changes */
  'playlist:changed': Playlist;
  /** WebSocket connection opened */
  'ws:open': undefined;
  /** WebSocket connection closed */
  'ws:close': undefined;
  /** WebSocket or subscription error */
  'ws:error': Error;
}

type EventListener<T> = T extends undefined ? () => void : (payload: T) => void;

// ---------------------------------------------------------------------------
// Minimal typed EventEmitter — no Node.js dependency
// ---------------------------------------------------------------------------

// eslint-disable-next-line @typescript-eslint/no-explicit-any
export class TypedEventEmitter<Events extends Record<string, any>> {
  private listeners = new Map<keyof Events, Set<(payload: unknown) => void>>();

  on<K extends keyof Events>(event: K, listener: EventListener<Events[K]>): this {
    if (!this.listeners.has(event)) this.listeners.set(event, new Set());
    this.listeners.get(event)!.add(listener as (payload: unknown) => void);
    return this;
  }

  once<K extends keyof Events>(event: K, listener: EventListener<Events[K]>): this {
    const wrapped = (payload: unknown) => {
      this.off(event, wrapped as EventListener<Events[K]>);
      (listener as (payload: unknown) => void)(payload);
    };
    return this.on(event, wrapped as EventListener<Events[K]>);
  }

  off<K extends keyof Events>(event: K, listener: EventListener<Events[K]>): this {
    this.listeners.get(event)?.delete(listener as (payload: unknown) => void);
    return this;
  }

  emit<K extends keyof Events>(
    event: K,
    ...args: Events[K] extends undefined ? [] : [Events[K]]
  ): void {
    this.listeners.get(event)?.forEach((fn) => fn(args[0]));
  }

  removeAllListeners(event?: keyof Events): this {
    if (event) {
      this.listeners.delete(event);
    } else {
      this.listeners.clear();
    }
    return this;
  }
}
