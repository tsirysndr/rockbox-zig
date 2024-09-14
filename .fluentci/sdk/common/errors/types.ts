import { GraphQLError } from "npm:graphql@16.8.1";

export interface GraphQLResponse<T = unknown> {
  data?: T;
  errors?: GraphQLError[];
  extensions?: unknown;
  status: number;
  [key: string]: unknown;
}

export type Variables = Record<string, unknown>;

export interface GraphQLRequestContext<V extends Variables = Variables> {
  query: string | string[];
  variables?: V;
}
