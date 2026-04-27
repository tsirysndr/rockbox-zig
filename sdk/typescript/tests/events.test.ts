import { describe, it, expect, vi } from 'vitest';
import { TypedEventEmitter } from '../src/events.js';
import type { RockboxEventMap } from '../src/events.js';

describe('TypedEventEmitter', () => {
  it('calls registered listener with the correct payload', () => {
    const emitter = new TypedEventEmitter<RockboxEventMap>();
    const handler = vi.fn();
    emitter.on('status:changed', handler);
    emitter.emit('status:changed', 1);
    expect(handler).toHaveBeenCalledWith(1);
  });

  it('supports multiple listeners for the same event', () => {
    const emitter = new TypedEventEmitter<RockboxEventMap>();
    const a = vi.fn();
    const b = vi.fn();
    emitter.on('status:changed', a).on('status:changed', b);
    emitter.emit('status:changed', 2);
    expect(a).toHaveBeenCalledWith(2);
    expect(b).toHaveBeenCalledWith(2);
  });

  it('removes a listener with off()', () => {
    const emitter = new TypedEventEmitter<RockboxEventMap>();
    const handler = vi.fn();
    emitter.on('status:changed', handler);
    emitter.off('status:changed', handler);
    emitter.emit('status:changed', 1);
    expect(handler).not.toHaveBeenCalled();
  });

  it('once() fires exactly once', () => {
    const emitter = new TypedEventEmitter<RockboxEventMap>();
    const handler = vi.fn();
    emitter.once('status:changed', handler);
    emitter.emit('status:changed', 1);
    emitter.emit('status:changed', 2);
    expect(handler).toHaveBeenCalledTimes(1);
    expect(handler).toHaveBeenCalledWith(1);
  });

  it('removeAllListeners() clears all handlers for a given event', () => {
    const emitter = new TypedEventEmitter<RockboxEventMap>();
    const a = vi.fn();
    const b = vi.fn();
    emitter.on('status:changed', a).on('status:changed', b);
    emitter.removeAllListeners('status:changed');
    emitter.emit('status:changed', 1);
    expect(a).not.toHaveBeenCalled();
    expect(b).not.toHaveBeenCalled();
  });

  it('does not call handlers for a different event', () => {
    const emitter = new TypedEventEmitter<RockboxEventMap>();
    const handler = vi.fn();
    emitter.on('status:changed', handler);
    emitter.emit('ws:close');
    expect(handler).not.toHaveBeenCalled();
  });
});
