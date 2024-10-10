import { FC } from "react";
import ArtistDetails from "./ArtistDetails";
import { tracks, albums } from "./mocks";
import { useNavigate } from "react-router-dom";

const ArtistDetailsWithData: FC = () => {
  const navigate = useNavigate();
  return (
    <ArtistDetails
      name={"Daft Punk"}
      tracks={tracks}
      albums={albums}
      // eslint-disable-next-line @typescript-eslint/no-unused-vars
      onClickAlbum={(_album) => {}}
      onPlayAll={() => {}}
      onShuffleAll={() => {}}
      onPlayAlbum={() => {}}
      onLikeAlbum={() => {}}
      onUnLikeAlbum={() => {}}
      onLikeTrack={() => {}}
      onUnlikeTrack={() => {}}
      onGoBack={() => navigate(-1)}
    />
  );
};

export default ArtistDetailsWithData;
