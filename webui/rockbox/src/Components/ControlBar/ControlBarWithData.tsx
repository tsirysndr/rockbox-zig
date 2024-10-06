import { FC } from "react";
import ControlBar from "./ControlBar";

const ControlBarWithData: FC = () => {
  return (
    <ControlBar
      nowPlaying={{
        album: "Pluto x Baby Pluto (Deluxe)",
        artist: "Future, Lil Uzi Vert",
        title: "Drankin N Smokin",
        cover:
          "https://resources.tidal.com/images/fe6787d5/4ba5/4d3e/8576/48943ee6a768/320x320.jpg",
        duration: 255488.00659179688,
        progress: 123456.789,
        isPlaying: true,
        albumId: "229251493",
      }}
      onPlay={() => {}}
      onPause={() => {}}
      onNext={() => {}}
      onPrevious={() => {}}
      onShuffle={() => {}}
      onRepeat={() => {}}
    />
  );
};

export default ControlBarWithData;
