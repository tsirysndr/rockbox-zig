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
      artistId
      md5
      tracks {
        id
        title
        artist
        album
        albumArtist
        artistId
        albumId
        path
        length
      }
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
      tracknum
      title
      artist
      album
      albumArtist
      artistId
      albumId
      path
      length
    }
  }
`;

export const GET_ALBUM = gql`
  query GetAlbum($id: String!) {
    album(id: $id) {
      id
      title
      artist
      albumArt
      year
      yearString
      artistId
      md5
      tracks {
        id
        title
        tracknum
        artist
        album
        albumArtist
        artistId
        albumId
        path
        length
      }
    }
  }
`;
