import { gql } from "@apollo/client";

export const PLAYLIST_CHANGED = gql`
  subscription PlaylistChanged {
    playlistChanged {
      index
      amount
      maxPlaylistSize
      tracks {
        id
        title
        artist
        albumArt
        artistId
        albumId
        path
      }
    }
  }
`;
