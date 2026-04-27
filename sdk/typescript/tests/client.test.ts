import { describe, it, expect, vi, beforeEach } from 'vitest';
import { RockboxClient } from '../src/client.js';

const mockFetch = vi.fn();
vi.stubGlobal('fetch', mockFetch);

function makeOk(data: unknown) {
  return {
    ok: true,
    status: 200,
    json: () => Promise.resolve({ data }),
  } as Response;
}

describe('RockboxClient', () => {
  beforeEach(() => mockFetch.mockReset());

  it('constructs with defaults (localhost:6062)', () => {
    const client = new RockboxClient();
    expect(client).toBeDefined();
    expect(client.playback).toBeDefined();
    expect(client.library).toBeDefined();
    expect(client.playlist).toBeDefined();
    expect(client.savedPlaylists).toBeDefined();
    expect(client.smartPlaylists).toBeDefined();
    expect(client.sound).toBeDefined();
    expect(client.settings).toBeDefined();
    expect(client.system).toBeDefined();
    expect(client.browse).toBeDefined();
    expect(client.devices).toBeDefined();
  });

  it('constructs with custom host and port', async () => {
    mockFetch.mockResolvedValue(makeOk({ rockboxVersion: '4.0' }));
    const client = new RockboxClient({ host: '192.168.1.42', port: 7070 });
    await client.system.version();
    const [url] = mockFetch.mock.calls[0] as [string, RequestInit];
    expect(url).toBe('http://192.168.1.42:7070/graphql');
  });

  it('query() is a raw escape hatch that calls the HTTP transport', async () => {
    mockFetch.mockResolvedValue(makeOk({ rockboxVersion: '4.0' }));
    const client = new RockboxClient();
    const result = await client.query<{ rockboxVersion: string }>('query { rockboxVersion }');
    expect(result.rockboxVersion).toBe('4.0');
  });

  it('installedPlugins() starts empty', () => {
    const client = new RockboxClient();
    expect(client.installedPlugins()).toHaveLength(0);
  });

  it('use() installs a plugin and calls its install hook', async () => {
    const client = new RockboxClient();
    const install = vi.fn();
    await client.use({ name: 'test-plugin', version: '0.1', install });
    expect(install).toHaveBeenCalledTimes(1);
    expect(client.installedPlugins()).toHaveLength(1);
  });

  it('is an event emitter — on/emit work', () => {
    const client = new RockboxClient();
    const handler = vi.fn();
    client.on('status:changed', handler);
    client.emit('status:changed', 1);
    expect(handler).toHaveBeenCalledWith(1);
  });
});
