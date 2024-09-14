import { Writable } from "node:stream";
import { Client } from "./client.gen.ts";
import { Context } from "./context.ts";

/**
 * ConnectOpts defines option used to connect to an engine.
 */
export interface ConnectOpts {
  /**
   * Use to overwrite Dagger workdir
   * @defaultValue process.cwd()
   */
  Workdir?: string;
  /**
   * Enable logs output
   * @example
   * LogOutput
   * ```ts
   * connect(async (client: Client) => {
    const source = await client.host().workdir().id()
    ...
    }, {LogOutput: process.stdout})
    ```
   */
  LogOutput?: Writable;
}

export type CallbackFct = (client: Client) => Promise<void>;

export interface ConnectParams {
  port: number;
  session_token: string;
}

/**
 * connect runs GraphQL server and initializes a
 * GraphQL client to execute query on it through its callback.
 * This implementation is based on the existing Go SDK.
 */
export async function connect(
  cb: CallbackFct,
  _config: ConnectOpts = {}
): Promise<void> {
  const ctx = new Context();
  const client = new Client({ ctx: ctx });

  // Initialize connection
  await ctx.connection();

  await cb(client).finally(() => {
    ctx.close();
  });
}
