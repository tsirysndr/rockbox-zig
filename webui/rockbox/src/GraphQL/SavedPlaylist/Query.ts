import { gql } from "graphql-tag";

export const GET_SAVED_PLAYLISTS = gql`
  query GetSavedPlaylists {
    savedPlaylists {
      id
      name
      description
      image
      trackCount
      createdAt
      updatedAt
    }
  }
`;

export const GET_SAVED_PLAYLIST = gql`
  query GetSavedPlaylist($id: String!) {
    savedPlaylist(id: $id) {
      id
      name
      description
      image
      trackCount
      createdAt
      updatedAt
    }
  }
`;

export const GET_SAVED_PLAYLIST_TRACKS = gql`
  query GetSavedPlaylistTracks($playlistId: String!) {
    savedPlaylistTracks(playlistId: $playlistId) {
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
