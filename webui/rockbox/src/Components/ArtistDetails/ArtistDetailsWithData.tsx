import { FC } from "react";
import ArtistDetails from "./ArtistDetails";
import { tracks, albums } from "./mocks";

const ArtistDetailsWithData: FC = () => {
  return (
    <ArtistDetails
      name={"Daft Punk"}
      tracks={tracks}
      albums={albums}
      // eslint-disable-next-line @typescript-eslint/no-unused-vars
      onClickAlbum={(_album) => {}}
      onPlayAll={() => {}}
      onShuffleAll={() => {}}
    />
  );
};

export default ArtistDetailsWithData;
