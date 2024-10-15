import { FC } from "react";
import PlayQueue from "./PlayQueue";

const PlayQueueWithData: FC = () => {
  return (
    <PlayQueue
      previousTracks={[]}
      nextTracks={[]}
      currentTrack={undefined}
      onPlayTrackAt={() => {}}
      onRemoveTrackAt={() => {}}
    />
  );
};

export default PlayQueueWithData;
