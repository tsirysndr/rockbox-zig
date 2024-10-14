import { gql } from "@apollo/client";

export const CURRENTLY_PLAYING_SONG = gql`
  subscription CurrentlyPlayingSong {
    currentlyPlayingSong {
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

export const PLAYBACK_STATUS = gql`
  subscription PlaybackStatus {
    playbackStatus {
      status
    }
  }
`;
