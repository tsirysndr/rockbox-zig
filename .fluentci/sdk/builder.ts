import { createGQLClient } from "./client.ts";
import { Context } from "./context.ts";
import { env } from "../deps.ts";

/**
 * @hidden
 *
 * Initialize a default client context from environment.
 */
export function initDefaultContext(): Context {
  let ctx = new Context();

  // Prefer DAGGER_SESSION_PORT if set
  const daggerSessionPort = env.get("DAGGER_SESSION_PORT");
  if (daggerSessionPort) {
    const sessionToken = env.get("DAGGER_SESSION_TOKEN");
    if (!sessionToken) {
      throw new Error(
        "DAGGER_SESSION_TOKEN must be set when using DAGGER_SESSION_PORT"
      );
    }

    ctx = new Context({
      client: createGQLClient(Number(daggerSessionPort), sessionToken),
    });
  } else {
    throw new Error("DAGGER_SESSION_PORT must be set");
  }

  return ctx;
}
