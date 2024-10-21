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

export const PLAY_ALBUM = gql`
  mutation PlayAlbum($albumId: String!, $shuffle: Boolean) {
    playAlbum(albumId: $albumId, shuffle: $shuffle)
  }
`;

export const PLAY_ARTIST_TRACKS = gql`
  mutation PlayArtistTracks($artistId: String!, $shuffle: Boolean) {
    playArtistTracks(artistId: $artistId, shuffle: $shuffle)
  }
`;

export const PLAY_DIRECTORY = gql`
  mutation PlayDirectory($path: String!, $recurse: Boolean, $shuffle: Boolean) {
    playDirectory(path: $path, recurse: $recurse, shuffle: $shuffle)
  }
`;

export const PLAY_TRACK = gql`
  mutation PlayTrack($path: String!) {
    playTrack(path: $path)
  }
`;

export const PLAY_LIKED_TRACKS = gql`
  mutation PlayLikedTracks($shuffle: Boolean) {
    playLikedTracks(shuffle: $shuffle)
  }
`;
