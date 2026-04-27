import { FC } from "react";
import ContextMenu from "./ContextMenu";
import {
  useGetLikedTracksQuery,
  useGetSavedPlaylistsQuery,
  useInsertTracksMutation,
  useLikeTrackMutation,
  useUnlikeTrackMutation,
  useAddTracksToSavedPlaylistMutation,
} from "../../Hooks/GraphQL";
import {
  PLAYLIST_INSERT_FIRST,
  PLAYLIST_INSERT_LAST,
  PLAYLIST_INSERT_SHUFFLED,
} from "../../Types/playlist";
import { useRecoilState } from "recoil";
import { likesState } from "../Likes/LikesState";

export type ContextMenuWithDataProps = {
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  track: any;
};

const ContextMenuWithData: FC<ContextMenuWithDataProps> = ({ track }) => {
  const [likes, setLikes] = useRecoilState(likesState);
  const { refetch } = useGetLikedTracksQuery({
    fetchPolicy: "network-only",
  });
  const { data: playlistsData } = useGetSavedPlaylistsQuery({
    fetchPolicy: "cache-and-network",
  });
  const [insertTracks] = useInsertTracksMutation();
  const [likeTrack] = useLikeTrackMutation();
  const [unlikeTrack] = useUnlikeTrackMutation();
  const [addTracksToPlaylist] = useAddTracksToSavedPlaylistMutation();

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

  const onLike = async (trackId: string) => {
    setLikes({
      ...likes,
      [trackId]: true,
    });
    await likeTrack({
      variables: {
        trackId,
      },
    });
    await refetch();
  };

  const onUnlike = async (trackId: string) => {
    setLikes({
      ...likes,
      [trackId]: false,
    });
    await unlikeTrack({
      variables: {
        trackId,
      },
    });
    await refetch();
  };

  const onAddTrackToPlaylist = (playlistId: string, trackId: string) => {
    addTracksToPlaylist({
      variables: {
        playlistId,
        trackIds: [trackId],
      },
    });
  };

  return (
    <ContextMenu
      track={track}
      onPlayNext={onPlayNext}
      onCreatePlaylist={() => {}}
      onAddTrackToPlaylist={onAddTrackToPlaylist}
      onPlayLast={onPlayLast}
      onAddShuffled={onAddShuffled}
      onLike={onLike}
      onUnlike={onUnlike}
      recentPlaylists={playlistsData?.savedPlaylists ?? []}
      liked={likes[track.id]}
    />
  );
};

export default ContextMenuWithData;
