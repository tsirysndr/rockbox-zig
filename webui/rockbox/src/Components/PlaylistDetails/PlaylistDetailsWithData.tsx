import { FC, useEffect, useMemo, useState } from "react";
import { useNavigate, useParams } from "react-router-dom";
import {
  useGetSavedPlaylistQuery,
  useGetSavedPlaylistTracksQuery,
  useGetSmartPlaylistQuery,
  useGetSmartPlaylistTracksQuery,
  usePlaySavedPlaylistMutation,
  usePlaySmartPlaylistMutation,
  useRemoveTrackFromSavedPlaylistMutation,
  useInsertTracksMutation,
} from "../../Hooks/GraphQL";
import { useTimeFormat } from "../../Hooks/useFormat";
import { Track } from "../../Types/track";
import { PLAYLIST_INSERT_FIRST } from "../../Types/playlist";
import PlaylistDetails from "./PlaylistDetails";

type Props = { isSmart?: boolean };

const PlaylistDetailsWithData: FC<Props> = ({ isSmart = false }) => {
  const { id } = useParams<{ id: string }>();
  const navigate = useNavigate();
  const { formatTime } = useTimeFormat();
  const [tracks, setTracks] = useState<Track[]>([]);

  const { data: savedData } = useGetSavedPlaylistQuery({
    variables: { id: id! },
    skip: isSmart,
  });
  const { data: savedTracksData, refetch: refetchTracks } =
    useGetSavedPlaylistTracksQuery({
      variables: { playlistId: id! },
      skip: isSmart,
    });

  const { data: smartData } = useGetSmartPlaylistQuery({
    variables: { id: id! },
    skip: !isSmart,
  });
  const { data: smartTracksData } = useGetSmartPlaylistTracksQuery({
    variables: { id: id! },
    skip: !isSmart,
  });

  const [playSaved] = usePlaySavedPlaylistMutation();
  const [playSmart] = usePlaySmartPlaylistMutation();
  const [removeTrack] = useRemoveTrackFromSavedPlaylistMutation();
  const [insertTracks] = useInsertTracksMutation();

  const playlist = useMemo(
    () => (isSmart ? smartData?.smartPlaylist : savedData?.savedPlaylist),
    [isSmart, savedData, smartData]
  );

  const rawTracks = useMemo(
    () =>
      isSmart
        ? smartTracksData?.smartPlaylistTracks
        : savedTracksData?.savedPlaylistTracks,
    [isSmart, savedTracksData, smartTracksData]
  );

  useEffect(() => {
    if (!rawTracks) return;
    setTracks(
      rawTracks.map((t, i) => ({
        id: t.id ?? "",
        trackNumber: i + 1,
        title: t.title,
        artist: t.artist,
        artistId: t.artistId ?? undefined,
        albumId: t.albumId ?? undefined,
        time: formatTime(t.length),
        albumArt: t.albumArt
          ? `${location.protocol}//${location.host}/covers/${t.albumArt}`
          : undefined,
        path: t.path,
      }))
    );
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [rawTracks]);

  function onPlayAll(shuffle = false) {
    if (isSmart) {
      playSmart({ variables: { id: id! } });
    } else {
      if (shuffle) {
        const paths = tracks.map((t) => t.path).filter((p): p is string => !!p);
        const shuffled = [...paths].sort(() => Math.random() - 0.5);
        insertTracks({
          variables: { position: PLAYLIST_INSERT_FIRST, tracks: shuffled },
        });
      } else {
        playSaved({ variables: { playlistId: id! } });
      }
    }
  }

  function onPlayTrack(position: number) {
    if (isSmart) {
      playSmart({ variables: { id: id! } });
    } else {
      const paths = tracks.map((t) => t.path).filter((p): p is string => !!p);
      const ordered = [...paths.slice(position), ...paths.slice(0, position)];
      insertTracks({
        variables: { position: PLAYLIST_INSERT_FIRST, tracks: ordered },
      });
    }
  }

  async function onRemoveTrack(trackId: string) {
    await removeTrack({
      variables: { playlistId: id!, trackId },
    });
    await refetchTracks();
  }

  return (
    <PlaylistDetails
      playlist={playlist}
      tracks={tracks}
      isSmart={isSmart}
      onGoBack={() => navigate(-1)}
      onPlayAll={() => onPlayAll(false)}
      onShuffleAll={() => onPlayAll(true)}
      onPlayTrack={onPlayTrack}
      onRemoveTrack={!isSmart ? onRemoveTrack : undefined}
    />
  );
};

export default PlaylistDetailsWithData;
