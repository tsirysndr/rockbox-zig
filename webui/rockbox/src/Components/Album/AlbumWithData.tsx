/* eslint-disable @typescript-eslint/no-explicit-any */
import { FC } from "react";
import Album from "./Album";
import { usePlayAlbumMutation } from "../../Hooks/GraphQL";

export type AlbumWithDataProps = {
  album: any;
};

const AlbumWithData: FC<AlbumWithDataProps> = ({ album }) => {
  const [playAlbum] = usePlayAlbumMutation();

  const onPlay = ({ id: albumId }: any) => {
    playAlbum({
      variables: {
        albumId,
      },
    });
  };
  return (
    <Album
      album={album}
      onLike={() => {}}
      onPlay={onPlay}
      onUnLike={() => {}}
    />
  );
};

export default AlbumWithData;
