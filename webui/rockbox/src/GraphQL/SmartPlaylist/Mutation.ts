import { gql } from "@apollo/client";

export const PLAY_SMART_PLAYLIST = gql`
  mutation PlaySmartPlaylist($id: String!) {
    playSmartPlaylist(id: $id)
  }
`;
