import { gql } from "@apollo/client";

export const GET_ROCKBOX_VERSION = gql`
  query GetRockboxVersion {
    rockboxVersion
  }
`;
