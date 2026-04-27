import { gql } from "@apollo/client";

export const GET_SMART_PLAYLISTS = gql`
  query GetSmartPlaylists {
    smartPlaylists {
      id
      name
      description
      image
      isSystem
      createdAt
      updatedAt
    }
  }
`;

export const GET_SMART_PLAYLIST = gql`
  query GetSmartPlaylist($id: String!) {
    smartPlaylist(id: $id) {
      id
      name
      description
      image
      isSystem
      createdAt
      updatedAt
    }
  }
`;

export const GET_SMART_PLAYLIST_TRACKS = gql`
  query GetSmartPlaylistTracks($id: String!) {
    smartPlaylistTracks(id: $id) {
      id
      title
      artist
      album
      albumArt
      artistId
      albumId
      path
      length
      tracknum
    }
  }
`;
