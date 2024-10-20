/* eslint-disable @typescript-eslint/no-explicit-any */
import { FC } from "react";
import Album from "./Album";
import {
  useLikeAlbumMutation,
  usePlayAlbumMutation,
  useUnlikeAlbumMutation,
} from "../../Hooks/GraphQL";
import { useRecoilState } from "recoil";
import { likesState } from "../Likes/LikesState";

export type AlbumWithDataProps = {
  album: any;
};

const AlbumWithData: FC<AlbumWithDataProps> = ({ album }) => {
  const [likes, setLikes] = useRecoilState(likesState);
  const [playAlbum] = usePlayAlbumMutation();
  const [likeAlbum] = useLikeAlbumMutation();
  const [unlikeAlbum] = useUnlikeAlbumMutation();

  const onPlay = ({ id: albumId }: any) => {
    playAlbum({
      variables: {
        albumId,
      },
    });
  };

  const onLike = ({ id: albumId }: any) => {
    setLikes({
      ...likes,
      [albumId]: true,
    });
    likeAlbum({
      variables: {
        albumId,
      },
    });
  };

  const onUnlike = ({ id: albumId }: any) => {
    setLikes({
      ...likes,
      [albumId]: false,
    });
    unlikeAlbum({
      variables: {
        albumId,
      },
    });
  };

  return (
    <Album
      album={album}
      onLike={onLike}
      onPlay={onPlay}
      onUnLike={onUnlike}
      liked={likes[album.id]}
    />
  );
};

export default AlbumWithData;
