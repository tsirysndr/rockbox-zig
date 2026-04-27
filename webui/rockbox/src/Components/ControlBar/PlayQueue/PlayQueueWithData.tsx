import { FC } from "react";
import PlayQueue from "./PlayQueue";
import { usePlayQueue } from "../../../Hooks/usePlayQueue";
import {
  usePlaylistRemoveTrackMutation,
  useStartPlaylistMutation,
} from "../../../Hooks/GraphQL";

const PlayQueueWithData: FC = () => {
  const { nextTracks, previousTracks } = usePlayQueue();
  const { mutate: removeTrack } = usePlaylistRemoveTrackMutation();
  const { mutate: startPlaylist } = useStartPlaylistMutation();

  const onPlayTrackAt = (startIndex: number) => {
    startPlaylist({ startIndex });
  };

  const onRemoveTrackAt = (index: number) => {
    removeTrack({ index });
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
