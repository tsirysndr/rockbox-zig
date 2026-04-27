import { FC } from "react";
import ContextMenu from "./ContextMenu";
import {
  useInsertDirectoryMutation,
  useInsertTracksMutation,
  usePlayDirectoryMutation,
  useGetSavedPlaylistsQuery,
  useAddTracksToSavedPlaylistMutation,
  useCreateSavedPlaylistMutation,
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
  const [insertTracks] = useInsertTracksMutation();
  const [playDirectory] = usePlayDirectoryMutation();
  const [insertDirectory] = useInsertDirectoryMutation();
  const [addTracksToPlaylist] = useAddTracksToSavedPlaylistMutation();
  const [createPlaylist] = useCreateSavedPlaylistMutation();
  const { data: playlistsData } = useGetSavedPlaylistsQuery({
    fetchPolicy: "cache-and-network",
  });

  const onPlayNext = (path: string) => {
    if (!entry.isDirectory) {
      insertTracks({ variables: { tracks: [path], position: PLAYLIST_INSERT_FIRST } });
      return;
    }
    insertDirectory({ variables: { position: PLAYLIST_INSERT_FIRST, directory: path } });
  };

  const onPlayLast = (path: string) => {
    if (!entry.isDirectory) {
      insertTracks({ variables: { tracks: [path], position: PLAYLIST_INSERT_LAST } });
      return;
    }
    insertDirectory({ variables: { position: PLAYLIST_INSERT_LAST, directory: path } });
  };

  const onAddShuffled = (path: string) => {
    if (!entry.isDirectory) {
      insertTracks({ variables: { tracks: [path], position: PLAYLIST_INSERT_SHUFFLED } });
      return;
    }
    insertDirectory({ variables: { position: PLAYLIST_INSERT_SHUFFLED, directory: path } });
  };

  const onPlayShuffled = (path: string) => {
    playDirectory({ variables: { path, shuffle: true, recurse: true } });
  };

  const onPlayLastShuffled = (path: string) => {
    insertDirectory({ variables: { position: PLAYLIST_INSERT_LAST_SHUFFLED, directory: path } });
  };

  const onAddTrackToPlaylist = (playlistId: string, trackPath: string) => {
    addTracksToPlaylist({ variables: { playlistId, trackIds: [trackPath] } });
  };

  const onCreatePlaylist = async (name: string, trackPath: string, description?: string) => {
    await createPlaylist({ variables: { name, description, trackIds: [trackPath] } });
  };

  return (
    <ContextMenu
      entry={entry}
      onPlayNext={onPlayNext}
      onCreatePlaylist={onCreatePlaylist}
      onAddTrackToPlaylist={onAddTrackToPlaylist}
      onPlayLast={onPlayLast}
      onAddShuffled={onAddShuffled}
      onPlayShuffled={onPlayShuffled}
      onPlayLastShuffled={onPlayLastShuffled}
      recentPlaylists={playlistsData?.savedPlaylists ?? []}
    />
  );
};

export default ContextMenuWithData;
