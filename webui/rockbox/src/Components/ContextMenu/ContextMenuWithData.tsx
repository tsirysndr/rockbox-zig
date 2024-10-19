import { FC } from "react";
import ContextMenu from "./ContextMenu";
import { useInsertTracksMutation } from "../../Hooks/GraphQL";
import {
  PLAYLIST_INSERT_FIRST,
  PLAYLIST_INSERT_LAST,
  PLAYLIST_INSERT_SHUFFLED,
} from "../../Types/playlist";

export type ContextMenuWithDataProps = {
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  track: any;
};

const ContextMenuWithData: FC<ContextMenuWithDataProps> = ({ track }) => {
  const [insertTracks] = useInsertTracksMutation();

  const onPlayNext = (path: string) => {
    insertTracks({
      variables: {
        position: PLAYLIST_INSERT_FIRST,
        tracks: [path],
      },
    });
  };

  const onPlayLast = (path: string) => {
    insertTracks({
      variables: {
        position: PLAYLIST_INSERT_LAST,
        tracks: [path],
      },
    });
  };

  const onAddShuffled = (path: string) => {
    insertTracks({
      variables: {
        position: PLAYLIST_INSERT_SHUFFLED,
        tracks: [path],
      },
    });
  };

  return (
    <ContextMenu
      track={track}
      onPlayNext={onPlayNext}
      onCreatePlaylist={() => {}}
      onAddTrackToPlaylist={() => {}}
      onPlayLast={onPlayLast}
      onAddShuffled={onAddShuffled}
      recentPlaylists={[]}
    />
  );
};

export default ContextMenuWithData;
