import { FC } from "react";
import Tracks from "./Tracks";
import { tracks } from "./mocks";

const TracksWithData: FC = () => {
  return <Tracks tracks={tracks} />;
};

export default TracksWithData;
