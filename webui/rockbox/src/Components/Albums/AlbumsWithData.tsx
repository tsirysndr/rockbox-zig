import { FC } from "react";
import Albums from "./Albums";
import { albums } from "./mocks";

const AlbumsWithData: FC = () => {
  return (
    <Albums
      onClickAlbum={() => {}}
      onFilter={() => {}}
      albums={albums}
      onLike={() => {}}
      onPlay={() => {}}
      onUnLike={() => {}}
    />
  );
};

export default AlbumsWithData;
