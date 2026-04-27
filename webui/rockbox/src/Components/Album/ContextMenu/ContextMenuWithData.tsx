import { FC } from "react";
import ContextMenu from "./ContextMenu";
import {
  useInsertAlbumMutation,
  usePlayAlbumMutation,
  useGetSavedPlaylistsQuery,
  useAddTracksToSavedPlaylistMutation,
  useCreateSavedPlaylistMutation,
  useGetAlbumLazyQuery,
} from "../../../Hooks/GraphQL";
import {
  PLAYLIST_INSERT_FIRST,
  PLAYLIST_INSERT_LAST,
  PLAYLIST_INSERT_LAST_SHUFFLED,
  PLAYLIST_INSERT_SHUFFLED,
} from "../../../Types/playlist";

export type ContextMenuWithDataProps = {
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  item: any;
};

const ContextMenuWithData: FC<ContextMenuWithDataProps> = ({ item }) => {
  const [playAlbum] = usePlayAlbumMutation();
  const [insertAlbum] = useInsertAlbumMutation();
  const [addTracksToPlaylist] = useAddTracksToSavedPlaylistMutation();
  const [createPlaylist] = useCreateSavedPlaylistMutation();
  const [getAlbum] = useGetAlbumLazyQuery();
  const { data: playlistsData } = useGetSavedPlaylistsQuery({
    fetchPolicy: "cache-and-network",
  });

  const onPlayShuffled = (albumId: string) => {
    playAlbum({ variables: { albumId, shuffle: true } });
  };

  const onPlayNext = (albumId: string) => {
    insertAlbum({ variables: { albumId, position: PLAYLIST_INSERT_FIRST } });
  };

  const onPlayLast = (albumId: string) => {
    insertAlbum({ variables: { albumId, position: PLAYLIST_INSERT_LAST } });
  };

  const onAddShuffled = (albumId: string) => {
    insertAlbum({ variables: { albumId, position: PLAYLIST_INSERT_SHUFFLED } });
  };

  const onPlayLastShuffled = (albumId: string) => {
    insertAlbum({ variables: { albumId, position: PLAYLIST_INSERT_LAST_SHUFFLED } });
  };

  const getAlbumTrackPaths = async (albumId: string): Promise<string[]> => {
    const result = await getAlbum({ variables: { id: albumId } });
    return (result.data?.album?.tracks ?? []).map((t) => t.path).filter(Boolean) as string[];
  };

  const onAddAlbumToPlaylist = async (playlistId: string, albumId: string) => {
    const paths = await getAlbumTrackPaths(albumId);
    if (paths.length > 0) {
      addTracksToPlaylist({ variables: { playlistId, trackIds: paths } });
    }
  };

  const onCreatePlaylist = async (name: string, albumId: string, description?: string) => {
    const paths = await getAlbumTrackPaths(albumId);
    await createPlaylist({ variables: { name, description, trackIds: paths } });
  };

  return (
    <ContextMenu
      album={item}
      onPlayNext={onPlayNext}
      onCreatePlaylist={onCreatePlaylist}
      onAddAlbumToPlaylist={onAddAlbumToPlaylist}
      onPlayLast={onPlayLast}
      onAddShuffled={onAddShuffled}
      onPlayLastShuffled={onPlayLastShuffled}
      onPlayShuffled={onPlayShuffled}
      recentPlaylists={playlistsData?.savedPlaylists ?? []}
    />
  );
};

export default ContextMenuWithData;
