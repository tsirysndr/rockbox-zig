import { gql } from "@apollo/client";

export const GET_ALBUMS = gql`
  query GetAlbums {
    albums {
      id
      title
      artist
      albumArt
      year
      yearString
    }
  }
`;

export const GET_ARTISTS = gql`
  query GetArtists {
    artists {
      id
      name
    }
  }
`;

export const GET_TRACKS = gql`
  query Tracks {
    tracks {
      id
      title
      artist
      albumArtist
      album
      path
      bitrate
      frequency
      filesize
      length
    }
  }
`;
