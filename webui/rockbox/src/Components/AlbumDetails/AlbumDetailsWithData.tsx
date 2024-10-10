import { FC } from "react";
import AlbumDetails from "./AlbumDetails";
import { tracks } from "./mocks";

const AlbumDetailsWithData: FC = () => {
  return (
    <AlbumDetails
      onGoBack={() => {}}
      onLike={() => {}}
      onPlayAll={() => {}}
      onShuffleAll={() => {}}
      onUnlike={() => {}}
      tracks={tracks}
    />
  );
};

export default AlbumDetailsWithData;
