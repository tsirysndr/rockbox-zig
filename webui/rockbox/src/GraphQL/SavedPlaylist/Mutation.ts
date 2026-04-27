import { gql } from "@apollo/client";

export const CREATE_SAVED_PLAYLIST = gql`
  mutation CreateSavedPlaylist(
    $name: String!
    $description: String
    $trackIds: [String!]
  ) {
    createSavedPlaylist(name: $name, description: $description, trackIds: $trackIds) {
      id
      name
      description
      trackCount
    }
  }
`;

export const UPDATE_SAVED_PLAYLIST = gql`
  mutation UpdateSavedPlaylist(
    $id: String!
    $name: String!
    $description: String
  ) {
    updateSavedPlaylist(id: $id, name: $name, description: $description)
  }
`;

export const DELETE_SAVED_PLAYLIST = gql`
  mutation DeleteSavedPlaylist($id: String!) {
    deleteSavedPlaylist(id: $id)
  }
`;

export const ADD_TRACKS_TO_SAVED_PLAYLIST = gql`
  mutation AddTracksToSavedPlaylist($playlistId: String!, $trackIds: [String!]!) {
    addTracksToSavedPlaylist(playlistId: $playlistId, trackIds: $trackIds)
  }
`;

export const REMOVE_TRACK_FROM_SAVED_PLAYLIST = gql`
  mutation RemoveTrackFromSavedPlaylist($playlistId: String!, $trackId: String!) {
    removeTrackFromSavedPlaylist(playlistId: $playlistId, trackId: $trackId)
  }
`;

export const PLAY_SAVED_PLAYLIST = gql`
  mutation PlaySavedPlaylist($playlistId: String!) {
    playSavedPlaylist(playlistId: $playlistId)
  }
`;
