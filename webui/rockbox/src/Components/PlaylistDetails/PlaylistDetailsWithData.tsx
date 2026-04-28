import { FC, useMemo } from "react";
import { useNavigate, useParams } from "react-router-dom";
import { useQueryClient } from "@tanstack/react-query";
import PlaylistDetails from "./PlaylistDetails";
import {
  useGetSavedPlaylistQuery,
  useGetSavedPlaylistTracksQuery,
  useGetSmartPlaylistQuery,
  useGetSmartPlaylistTracksQuery,
  usePlaySavedPlaylistMutation,
  usePlaySmartPlaylistMutation,
  useShufflePlaylistMutation,
  useStartPlaylistMutation,
  useRemoveTrackFromSavedPlaylistMutation,
} from "../../Hooks/GraphQL";
import { useTimeFormat } from "../../Hooks/useFormat";
import { Track } from "../../Types/track";

type Props = { isSmart?: boolean };

const PlaylistDetailsWithData: FC<Props> = ({ isSmart = false }) => {
  const navigate = useNavigate();
  const { id } = useParams<{ id: string }>();
  const { formatTime } = useTimeFormat();
  const queryClient = useQueryClient();

  const { data: savedPlaylistData, isLoading: savedPlaylistLoading } = useGetSavedPlaylistQuery(
    { id: id! },
    { enabled: !isSmart }
  );
  const { data: savedTracksData, isLoading: savedTracksLoading, refetch: refetchSavedTracks } =
    useGetSavedPlaylistTracksQuery(
      { playlistId: id! },
      { enabled: !isSmart }
    );

  const { data: smartPlaylistData, isLoading: smartPlaylistLoading } = useGetSmartPlaylistQuery(
    { id: id! },
    { enabled: isSmart }
  );
  const { data: smartTracksData, isLoading: smartTracksLoading } = useGetSmartPlaylistTracksQuery(
    { id: id! },
    { enabled: isSmart }
  );

  const isLoading = isSmart
    ? smartPlaylistLoading || smartTracksLoading
    : savedPlaylistLoading || savedTracksLoading;

  const { mutate: playSavedPlaylist } = usePlaySavedPlaylistMutation();
  const { mutate: playSmartPlaylist } = usePlaySmartPlaylistMutation();
  const { mutate: shufflePlaylist } = useShufflePlaylistMutation();
  const { mutate: startPlaylist } = useStartPlaylistMutation();
  const { mutateAsync: removeTrack } = useRemoveTrackFromSavedPlaylistMutation();

  const playlist = isSmart
    ? smartPlaylistData?.smartPlaylist
    : savedPlaylistData?.savedPlaylist;

  const rawTracks = isSmart
    ? (smartTracksData?.smartPlaylistTracks ?? [])
    : (savedTracksData?.savedPlaylistTracks ?? []);

  const tracks: Track[] = useMemo(
    () =>
      rawTracks.map((t, i) => ({
        id: t.id ?? "",
        trackNumber: i + 1,
        title: t.title,
        artist: t.artist,
        artistId: t.artistId ?? "",
        albumId: t.albumId ?? "",
        album: t.album,
        time: formatTime(t.length),
        albumArt: t.albumArt ?? undefined,
        path: t.path,
      })),
    // eslint-disable-next-line react-hooks/exhaustive-deps
    [rawTracks]
  );

  const onPlayAll = () => {
    if (isSmart) {
      playSmartPlaylist({ id: id! });
    } else {
      playSavedPlaylist({ playlistId: id! });
    }
  };

  const onShuffleAll = () => {
    onPlayAll();
    shufflePlaylist({});
  };

  const onPlayTrack = (position: number) => {
    if (isSmart) {
      playSmartPlaylist({ id: id! });
    } else {
      playSavedPlaylist({ playlistId: id! });
    }
    startPlaylist({ startIndex: position });
  };

  const onRemoveTrack = isSmart
    ? undefined
    : async (trackId: string) => {
        await removeTrack({ playlistId: id!, trackId });
        refetchSavedTracks();
        queryClient.invalidateQueries({
          queryKey: useGetSavedPlaylistTracksQuery.getKey({ playlistId: id! }),
        });
      };

  return (
    <PlaylistDetails
      playlist={playlist}
      tracks={tracks}
      isSmart={isSmart}
      loading={isLoading}
      onGoBack={() => navigate(-1)}
      onPlayAll={onPlayAll}
      onShuffleAll={onShuffleAll}
      onPlayTrack={onPlayTrack}
      onRemoveTrack={onRemoveTrack}
    />
  );
};

export default PlaylistDetailsWithData;
