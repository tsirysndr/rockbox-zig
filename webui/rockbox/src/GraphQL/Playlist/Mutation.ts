import { gql } from "@apollo/client";

export const RESUME_PLAYLIST = gql`
  mutation ResumePlaylist {
    playlistResume
  }
`;

export const RESUME_PLAYLIST_TRACK = gql`
  mutation ResumePlaylistTrack {
    resumeTrack
  }
`;

export const PLAYLIST_REMOVE_TRACK = gql`
  mutation PlaylistRemoveTrack($index: Int!) {
    playlistRemoveTrack(index: $index)
  }
`;

export const START_PLAYLIST = gql`
  mutation StartPlaylist($startIndex: Int, $elapsed: Int, $offset: Int) {
    playlistStart(startIndex: $startIndex, elapsed: $elapsed, offset: $offset)
  }
`;
