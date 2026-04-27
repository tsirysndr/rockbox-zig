import { describe, it, expect, vi, beforeEach } from 'vitest';
import { HttpTransport } from '../src/transport.js';
import { RockboxGraphQLError, RockboxNetworkError } from '../src/errors.js';

const mockFetch = vi.fn();
vi.stubGlobal('fetch', mockFetch);

function makeResponse(body: unknown, status = 200) {
  return {
    ok: status >= 200 && status < 300,
    status,
    statusText: status === 200 ? 'OK' : 'Error',
    json: () => Promise.resolve(body),
  } as Response;
}

describe('HttpTransport', () => {
  const transport = new HttpTransport('http://localhost:6062/graphql');

  beforeEach(() => {
    mockFetch.mockReset();
  });

  it('returns data on successful response', async () => {
    mockFetch.mockResolvedValue(makeResponse({ data: { rockboxVersion: '4.0' } }));
    const result = await transport.execute<{ rockboxVersion: string }>('query { rockboxVersion }');
    expect(result).toEqual({ rockboxVersion: '4.0' });
  });

  it('passes variables as JSON body', async () => {
    mockFetch.mockResolvedValue(makeResponse({ data: { album: null } }));
    await transport.execute('query Album($id: String!) { album(id: $id) { id } }', { id: '123' });
    const body = JSON.parse((mockFetch.mock.calls[0]![1] as RequestInit).body as string);
    expect(body.variables).toEqual({ id: '123' });
  });

  it('throws RockboxGraphQLError when errors are present', async () => {
    mockFetch.mockResolvedValue(
      makeResponse({ data: null, errors: [{ message: 'not found' }] }),
    );
    await expect(transport.execute('query { albums { id } }')).rejects.toBeInstanceOf(
      RockboxGraphQLError,
    );
  });

  it('throws RockboxNetworkError on non-ok HTTP status', async () => {
    mockFetch.mockResolvedValue(makeResponse({}, 500));
    await expect(transport.execute('query { albums { id } }')).rejects.toBeInstanceOf(
      RockboxNetworkError,
    );
  });

  it('throws RockboxNetworkError when fetch rejects (connection refused)', async () => {
    mockFetch.mockRejectedValue(new TypeError('Failed to fetch'));
    await expect(transport.execute('query { albums { id } }')).rejects.toBeInstanceOf(
      RockboxNetworkError,
    );
  });
});
