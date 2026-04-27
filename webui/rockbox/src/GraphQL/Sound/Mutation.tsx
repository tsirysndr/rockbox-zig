import { gql } from "graphql-tag";

export const ADJUST_VOLUME = gql`
  mutation AdjustVolume($steps: Int!) {
    adjustVolume(steps: $steps)
  }
`;
