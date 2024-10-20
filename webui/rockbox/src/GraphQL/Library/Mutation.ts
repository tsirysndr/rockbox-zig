import { gql } from "@apollo/client";

export const LIKE_TRACK = gql`
  mutation LikeTrack($trackId: String!) {
    likeTrack(id: $trackId)
  }
`;

export const UNLIKE_TRACK = gql`
  mutation UnlikeTrack($trackId: String!) {
    unlikeTrack(id: $trackId)
  }
`;

export const LIKE_ALBUM = gql`
  mutation LikeAlbum($albumId: String!) {
    likeAlbum(id: $albumId)
  }
`;

export const UNLIKE_ALBUM = gql`
  mutation UnlikeAlbum($albumId: String!) {
    unlikeAlbum(id: $albumId)
  }
`;
