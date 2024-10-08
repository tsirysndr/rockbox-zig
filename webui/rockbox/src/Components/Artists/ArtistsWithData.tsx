import { FC } from "react";
import { artists } from "./mocks";
import Artists from "./Artists";

const ArtistsWithData: FC = () => {
  return (
    <Artists onFilter={() => {}} onClickArtist={() => {}} artists={artists} />
  );
};

export default ArtistsWithData;
