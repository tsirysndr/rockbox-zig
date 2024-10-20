import { gql } from "@apollo/client";

export const ADJUST_VOLUME = gql`
  mutation AdjustVolume($steps: Int!) {
    adjustVolume(steps: $steps)
  }
`;
