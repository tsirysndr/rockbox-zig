import { gql } from "@apollo/client";

export const GET_CURRENT_TRACK = gql`
  query GetCurrentTrack {
    currentTrack {
      id
      title
      artist
      album
      albumArt
      artistId
      albumId
      elapsed
      length
      year
      yearString
    }
  }
`;

export const GET_NEXT_TRACK = gql`
  query GetNextTrack {
    nextTrack {
      id
      title
      artist
      album
      albumArt
      artistId
      albumId
      length
      year
      yearString
    }
  }
`;

export const GET_PLAYBACK_STATUS = gql`
  query GetPlaybackStatus {
    status
  }
`;