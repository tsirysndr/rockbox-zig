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

export const INSERT_TRACKS = gql`
  mutation InsertTracks(
    $playlistId: String
    $position: Int!
    $tracks: [String!]!
  ) {
    insertTracks(playlistId: $playlistId, position: $position, tracks: $tracks)
  }
`;

export const INSERT_DIRECTORY = gql`
  mutation InsertDirectory(
    $playlistId: String
    $position: Int!
    $directory: String!
  ) {
    insertDirectory(
      playlistId: $playlistId
      position: $position
      directory: $directory
    )
  }
`;

export const INSERT_ALBUM = gql`
  mutation InsertAlbum($albumId: String!, $position: Int!) {
    insertAlbum(albumId: $albumId, position: $position)
  }
`;
