import { FC } from "react";
import ContextMenu from "./ContextMenu";
import {
  useGetLikedTracksQuery,
  useInsertTracksMutation,
  useLikeTrackMutation,
  useUnlikeTrackMutation,
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
  const { refetch } = useGetLikedTracksQuery();
  const { mutate: insertTracks } = useInsertTracksMutation();
  const { mutateAsync: likeTrackAsync } = useLikeTrackMutation();
  const { mutateAsync: unlikeTrackAsync } = useUnlikeTrackMutation();

  const onPlayNext = (path: string) => {
    insertTracks({ position: PLAYLIST_INSERT_FIRST, tracks: [path] });
  };

  const onPlayLast = (path: string) => {
    insertTracks({ position: PLAYLIST_INSERT_LAST, tracks: [path] });
  };

  const onAddShuffled = (path: string) => {
    insertTracks({ position: PLAYLIST_INSERT_SHUFFLED, tracks: [path] });
  };

  const onLike = async (trackId: string) => {
    setLikes({ ...likes, [trackId]: true });
    await likeTrackAsync({ trackId });
    await refetch();
  };

  const onUnlike = async (trackId: string) => {
    setLikes({ ...likes, [trackId]: false });
    await unlikeTrackAsync({ trackId });
    await refetch();
  };

  return (
    <ContextMenu
      track={track}
      onPlayNext={onPlayNext}
      onPlayLast={onPlayLast}
      onAddShuffled={onAddShuffled}
      onLike={onLike}
      onUnlike={onUnlike}
      recentPlaylists={[]}
      liked={likes[track.id]}
      onCreatePlaylist={() => {}}
      onAddTrackToPlaylist={() => {}}
    />
  );
};

export default ContextMenuWithData;
