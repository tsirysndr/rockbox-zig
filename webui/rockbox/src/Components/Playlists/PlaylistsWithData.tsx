import { FC } from "react";
import { useQueryClient } from "@tanstack/react-query";
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
  const queryClient = useQueryClient();
  const { data: savedData } = useGetSavedPlaylistsQuery();
  const { data: smartData } = useGetSmartPlaylistsQuery();
  const { mutateAsync: createSavedPlaylist } = useCreateSavedPlaylistMutation();
  const { mutateAsync: updateSavedPlaylist } = useUpdateSavedPlaylistMutation();
  const { mutateAsync: deleteSavedPlaylist } = useDeleteSavedPlaylistMutation();
  const { mutate: playSavedPlaylist } = usePlaySavedPlaylistMutation();
  const { mutate: playSmartPlaylist } = usePlaySmartPlaylistMutation();

  const savedPlaylists = savedData?.savedPlaylists ?? [];
  const smartPlaylists = smartData?.smartPlaylists ?? [];

  const onPlay = (id: string, isSmart: boolean) => {
    if (isSmart) {
      playSmartPlaylist({ id });
    } else {
      playSavedPlaylist({ playlistId: id });
    }
  };

  const onCreate = async (name: string, description?: string) => {
    await createSavedPlaylist({ name, description });
    queryClient.invalidateQueries({
      queryKey: useGetSavedPlaylistsQuery.getKey(),
    });
  };

  const onUpdate = async (id: string, name: string, description?: string) => {
    await updateSavedPlaylist({ id, name, description });
    queryClient.invalidateQueries({
      queryKey: useGetSavedPlaylistsQuery.getKey(),
    });
  };

  const onDelete = async (id: string) => {
    await deleteSavedPlaylist({ id });
    queryClient.invalidateQueries({
      queryKey: useGetSavedPlaylistsQuery.getKey(),
    });
  };

  return (
    <Playlists
      savedPlaylists={savedPlaylists}
      smartPlaylists={smartPlaylists}
      onPlay={onPlay}
      onEdit={() => {}}
      onDelete={onDelete}
      onCreate={onCreate}
      onUpdate={onUpdate}
    />
  );
};

export default PlaylistsWithData;
