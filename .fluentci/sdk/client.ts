import { GraphQLClient } from "../deps.ts";

export function createGQLClient(port: number, token: string): GraphQLClient {
  return new GraphQLClient(`http://127.0.0.1:${port}/query`, {
    headers: {
      Authorization: "Basic " + btoa(token + ":"),
    },
  });
}
