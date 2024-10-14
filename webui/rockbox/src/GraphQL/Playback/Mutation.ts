import { gql } from "@apollo/client";

export const PLAY = gql`
  mutation Play($elapsed: Int!, $offset: Int!) {
    play(elapsed: $elapsed, offset: $offset)
  }
`;

export const PAUSE = gql`
  mutation Pause {
    pause
  }
`;

export const RESUME = gql`
  mutation Resume {
    resume
  }
`;

export const PREVIOUS = gql`
  mutation Previous {
    previous
  }
`;

export const NEXT = gql`
  mutation Next {
    next
  }
`;
