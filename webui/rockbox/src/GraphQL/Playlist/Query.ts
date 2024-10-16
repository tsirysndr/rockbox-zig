import { gql } from "@apollo/client";

export const GET_CURRENT_PLAYLIST = gql`
  query GetCurrentPlaylist {
    playlistGetCurrent {
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
        album
        length
      }
    }
  }
`;
