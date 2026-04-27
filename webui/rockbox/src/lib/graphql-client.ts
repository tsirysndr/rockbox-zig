import { GraphQLClient } from "graphql-request";

const endpoint =
  process.env.NODE_ENV === "development"
    ? import.meta.env.VITE_APP_API_URL || "http://localhost:6062/graphql"
    : `${origin}/graphql`;

export const graphqlClient = new GraphQLClient(endpoint);

// eslint-disable-next-line @typescript-eslint/no-explicit-any
export class TypedDocumentString<TResult, TVariables extends Record<string, unknown> = Record<string, never>> extends String {
  __apiType?: TResult;
  __variablesType?: TVariables;
  constructor(
    private value: string,
    public __meta__?: Record<string, unknown>
  ) {
    super(value);
  }
  toString(): string {
    return this.value;
  }
}

export function fetchData<TData, TVariables>(
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  query: string | TypedDocumentString<any, any>,
  variables?: TVariables
): () => Promise<TData> {
  return async () =>
    graphqlClient.request<TData>(
      query.toString(),
      variables as Record<string, unknown>
    );
}
