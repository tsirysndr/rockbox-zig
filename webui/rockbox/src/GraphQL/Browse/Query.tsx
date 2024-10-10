import { gql } from "@apollo/client";

export const GET_ENTRIES = gql`
  query GetEntries($path: String!) {
    treeGetEntries(path: $path) {
      name
      timeWrite
    }
  }
`;
