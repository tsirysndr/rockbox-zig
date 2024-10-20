import { gql } from "@apollo/client";

export const GET_GLOBAL_SETTINGS = gql`
  query GetGlobalSettings {
    globalSettings {
      volume
      eqEnabled
      eqBandSettings {
        q
        cutoff
        gain
      }
    }
  }
`;
