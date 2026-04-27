import { createClient, type Client } from 'graphql-ws';
import { RockboxGraphQLError, RockboxNetworkError } from './errors.js';

export interface TransportConfig {
  httpUrl: string;
  wsUrl: string;
}

// ---------------------------------------------------------------------------
// HTTP transport — plain fetch, no extra deps
// ---------------------------------------------------------------------------

export class HttpTransport {
  constructor(private url: string) {}

  async execute<T>(
    query: string,
    variables?: unknown,
  ): Promise<T> {
    let res: Response;
    try {
      res = await fetch(this.url, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json', Accept: 'application/json' },
        body: JSON.stringify({ query, variables }),
      });
    } catch (err) {
      throw new RockboxNetworkError(`Failed to reach Rockbox at ${this.url}`, err);
    }

    if (!res.ok) {
      throw new RockboxNetworkError(`HTTP ${res.status} ${res.statusText}`);
    }

    const json = (await res.json()) as { data?: T; errors?: unknown[] };
    if (json.errors?.length) {
      throw new RockboxGraphQLError(json.errors as never);
    }

    return json.data as T;
  }
}

// ---------------------------------------------------------------------------
// WebSocket transport — lazy, auto-reconnecting via graphql-ws
// ---------------------------------------------------------------------------

export class WsTransport {
  private _client: Client | null = null;
  private readonly wsUrl: string;

  constructor(wsUrl: string) {
    this.wsUrl = wsUrl;
  }

  private client(): Client {
    if (!this._client) {
      this._client = createClient({
        url: this.wsUrl,
        retryAttempts: Infinity,
        shouldRetry: () => true,
        retryWait: (attempt) =>
          new Promise((resolve) =>
            setTimeout(resolve, Math.min(1000 * 2 ** attempt, 30_000)),
          ),
      });
    }
    return this._client;
  }

  subscribe<T>(
    query: string,
    variables: unknown,
    sink: {
      next(result: { data?: T | null }): void;
      error(error: unknown): void;
      complete(): void;
    },
  ): () => void {
    // graphql-ws wraps results in FormattedExecutionResult which has the same {data?} shape
    return this.client().subscribe<T>(
      { query, variables: variables as Record<string, unknown> },
      sink as never,
    );
  }

  dispose(): void {
    this._client?.dispose();
    this._client = null;
  }
}
