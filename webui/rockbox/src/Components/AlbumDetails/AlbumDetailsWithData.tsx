import { FC } from "react";
import AlbumDetails from "./AlbumDetails";
import { tracks } from "./mocks";
import { useNavigate } from "react-router-dom";

const AlbumDetailsWithData: FC = () => {
  const navigate = useNavigate();
  return (
    <AlbumDetails
      onGoBack={() => navigate(-1)}
      onLike={() => {}}
      onPlayAll={() => {}}
      onShuffleAll={() => {}}
      onUnlike={() => {}}
      tracks={tracks}
    />
  );
};

export default AlbumDetailsWithData;
