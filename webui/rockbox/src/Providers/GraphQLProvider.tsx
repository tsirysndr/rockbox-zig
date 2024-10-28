import {
  ApolloClient,
  createHttpLink,
  InMemoryCache,
  ApolloProvider,
  split,
  from,
} from "@apollo/client";
import { removeTypenameFromVariables } from "@apollo/client/link/remove-typename";
import { WebSocketLink } from "@apollo/client/link/ws";
import { getMainDefinition } from "@apollo/client/utilities";
import { SubscriptionClient } from "subscriptions-transport-ws";
import { FC, ReactNode } from "react";

const uri =
  process.env.NODE_ENV === "development"
    ? import.meta.env.VITE_APP_API_URL || "http://localhost:6062/graphql"
    : `${origin}/graphql`;

const removeTypenameLink = removeTypenameFromVariables();

const httpLink = createHttpLink({
  uri,
});

const wsLink = new WebSocketLink(
  new SubscriptionClient(uri.replace("http", "ws"))
);

const splitLink = split(
  ({ query }) => {
    const definition = getMainDefinition(query);
    return (
      definition.kind === "OperationDefinition" &&
      definition.operation === "subscription"
    );
  },
  wsLink,
  from([removeTypenameLink, httpLink])
);
const link = splitLink;

const client = new ApolloClient({
  link,
  cache: new InMemoryCache(),
});

const GraphQLProvider: FC<{
  children: ReactNode;
}> = ({ children }) => {
  return <ApolloProvider client={client}>{children}</ApolloProvider>;
};

export default GraphQLProvider;
