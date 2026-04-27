import { gql } from "graphql-tag";

export const GET_ENTRIES = gql`
  query GetEntries($path: String) {
    treeGetEntries(path: $path) {
      name
      attr
      timeWrite
    }
  }
`;
