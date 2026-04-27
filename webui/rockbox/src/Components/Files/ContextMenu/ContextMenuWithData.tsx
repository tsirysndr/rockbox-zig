import { FC } from "react";
import ContextMenu from "./ContextMenu";
import {
  useInsertDirectoryMutation,
  useInsertTracksMutation,
  usePlayDirectoryMutation,
} from "../../../Hooks/GraphQL";
import {
  PLAYLIST_INSERT_FIRST,
  PLAYLIST_INSERT_LAST,
  PLAYLIST_INSERT_LAST_SHUFFLED,
  PLAYLIST_INSERT_SHUFFLED,
} from "../../../Types/playlist";

export type ContextMenuWithDataProps = {
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  entry: any;
};

const ContextMenuWithData: FC<ContextMenuWithDataProps> = ({ entry }) => {
  const { mutate: insertTracks } = useInsertTracksMutation();
  const { mutate: playDirectory } = usePlayDirectoryMutation();
  const { mutate: insertDirectory } = useInsertDirectoryMutation();

  const onPlayNext = (path: string) => {
    if (!entry.isDirectory) {
      insertTracks({ tracks: [path], position: PLAYLIST_INSERT_FIRST });
      return;
    }
    insertDirectory({ position: PLAYLIST_INSERT_FIRST, directory: path });
  };

  const onPlayLast = (path: string) => {
    if (!entry.isDirectory) {
      insertTracks({ tracks: [path], position: PLAYLIST_INSERT_LAST });
      return;
    }
    insertDirectory({ position: PLAYLIST_INSERT_LAST, directory: path });
  };

  const onAddShuffled = (path: string) => {
    if (!entry.isDirectory) {
      insertTracks({ tracks: [path], position: PLAYLIST_INSERT_SHUFFLED });
      return;
    }
    insertDirectory({ position: PLAYLIST_INSERT_SHUFFLED, directory: path });
  };

  const onPlayShuffled = (path: string) => {
    playDirectory({ path, shuffle: true, recurse: true });
  };

  const onPlayLastShuffled = (path: string) => {
    insertDirectory({ position: PLAYLIST_INSERT_LAST_SHUFFLED, directory: path });
  };

  return (
    <ContextMenu
      entry={entry}
      onPlayNext={onPlayNext}
      onPlayLast={onPlayLast}
      onAddShuffled={onAddShuffled}
      onPlayShuffled={onPlayShuffled}
      onPlayLastShuffled={onPlayLastShuffled}
      recentPlaylists={[]}
      onCreatePlaylist={() => {}}
      onAddTrackToPlaylist={() => {}}
    />
  );
};

export default ContextMenuWithData;
