import {
  ApolloClient,
  createHttpLink,
  InMemoryCache,
  ApolloProvider,
} from "@apollo/client";
import { FC, ReactNode } from "react";

const uri =
  process.env.NODE_ENV === "development"
    ? import.meta.env.VITE_APP_API_URL || "http://localhost:6062/graphql"
    : // eslint-disable-next-line no-restricted-globals
      `${origin}/graphql`;

const httpLink = createHttpLink({
  uri,
});

const client = new ApolloClient({
  link: httpLink,
  cache: new InMemoryCache(),
});

const GraphQLProvider: FC<{
  children: ReactNode;
}> = ({ children }) => {
  return <ApolloProvider client={client}>{children}</ApolloProvider>;
};

export default GraphQLProvider;
