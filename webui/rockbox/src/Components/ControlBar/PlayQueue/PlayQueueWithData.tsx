import { FC } from "react";
import PlayQueue from "./PlayQueue";
import { usePlayQueue } from "../../../Hooks/usePlayQueue";
import {
  usePlaylistRemoveTrackMutation,
  useStartPlaylistMutation,
} from "../../../Hooks/GraphQL";

const PlayQueueWithData: FC = () => {
  const { nextTracks, previousTracks } = usePlayQueue();
  const [removeTrack] = usePlaylistRemoveTrackMutation();
  const [startPlaylist] = useStartPlaylistMutation();

  const onPlayTrackAt = (startIndex: number) => {
    startPlaylist({
      variables: {
        startIndex,
      },
    });
  };

  const onRemoveTrackAt = (index: number) => {
    removeTrack({
      variables: {
        index,
      },
    });
  };

  return (
    <PlayQueue
      previousTracks={previousTracks}
      nextTracks={nextTracks}
      currentTrack={undefined}
      onPlayTrackAt={onPlayTrackAt}
      onRemoveTrackAt={onRemoveTrackAt}
    />
  );
};

export default PlayQueueWithData;
