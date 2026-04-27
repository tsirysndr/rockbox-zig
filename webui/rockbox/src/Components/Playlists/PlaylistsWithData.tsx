import { FC } from "react";
import Playlists from "./Playlists";
import {
  useGetSavedPlaylistsQuery,
  useGetSmartPlaylistsQuery,
  useCreateSavedPlaylistMutation,
  useUpdateSavedPlaylistMutation,
  useDeleteSavedPlaylistMutation,
  usePlaySavedPlaylistMutation,
  usePlaySmartPlaylistMutation,
} from "../../Hooks/GraphQL";

const PlaylistsWithData: FC = () => {
  const { data: savedData, refetch: refetchSaved } = useGetSavedPlaylistsQuery({
    fetchPolicy: "cache-and-network",
  });
  const { data: smartData } = useGetSmartPlaylistsQuery({
    fetchPolicy: "cache-and-network",
  });

  const [createPlaylist] = useCreateSavedPlaylistMutation();
  const [updatePlaylist] = useUpdateSavedPlaylistMutation();
  const [deletePlaylist] = useDeleteSavedPlaylistMutation();
  const [playSaved] = usePlaySavedPlaylistMutation();
  const [playSmart] = usePlaySmartPlaylistMutation();

  async function onCreate(name: string, description?: string) {
    await createPlaylist({ variables: { name, description } });
    await refetchSaved();
  }

  async function onUpdate(id: string, name: string, description?: string) {
    await updatePlaylist({ variables: { id, name, description } });
    await refetchSaved();
  }

  async function onDelete(id: string) {
    await deletePlaylist({ variables: { id } });
    await refetchSaved();
  }

  function onPlay(id: string, isSmart: boolean) {
    if (isSmart) {
      playSmart({ variables: { id } });
    } else {
      playSaved({ variables: { playlistId: id } });
    }
  }

  return (
    <Playlists
      savedPlaylists={savedData?.savedPlaylists ?? []}
      smartPlaylists={smartData?.smartPlaylists ?? []}
      onPlay={onPlay}
      onEdit={() => {}}
      onDelete={onDelete}
      onCreate={onCreate}
      onUpdate={onUpdate}
    />
  );
};

export default PlaylistsWithData;
