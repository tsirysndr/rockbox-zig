import { GraphQLClient } from "../deps.ts";

import { initDefaultContext } from "./builder.ts";

interface ContextConfig {
  client?: GraphQLClient;
}

/**
 * Context abstracts the connection to the engine.
 *
 * It's required to implement the default global SDK.
 * Its purpose is to store and returns the connection to the graphQL API, if
 * no connection is set, it can create its own.
 *
 * This is also useful for lazy evaluation with the default global client,
 * this one should only run the engine if it actually executes something.
 */
export class Context {
  private _client?: GraphQLClient;

  constructor(config?: ContextConfig) {
    this._client = config?.client;
  }

  /**
   * Returns a GraphQL client connected to the engine.
   *
   * If no client is set, it will create one.
   */
  public async connection(): Promise<GraphQLClient> {
    if (!this._client) {
      const defaultCtx = await initDefaultContext();
      this._client = defaultCtx._client as GraphQLClient;
    }

    return this._client;
  }

  /**
   * Close the connection and the engine if this one was started by the node
   * SDK.
   */
  public close(): void {
    // Reset client, so it can restart a new connection if necessary
    this._client = undefined;
  }
}

/**
 * Expose a default context for the global client
 */
export const defaultContext = new Context();
