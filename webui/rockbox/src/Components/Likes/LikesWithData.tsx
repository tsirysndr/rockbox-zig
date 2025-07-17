import { FC, useEffect } from "react";
import { useRecoilState, useRecoilValue } from "recoil";
import {
  useGetLikedTracksQuery,
  usePlayLikedTracksMutation,
} from "../../Hooks/GraphQL";
import { useTimeFormat } from "../../Hooks/useFormat";
import { filterState } from "../Filter/FilterState";
import Likes from "./Likes";
import { likedTracks, likesState } from "./LikesState";

const LikesWithData: FC = () => {
  const filter = useRecoilValue(filterState);
  const [likes, setLikes] = useRecoilState(likesState);
  const { data, loading } = useGetLikedTracksQuery({
    fetchPolicy: "network-only",
  });
  const [tracks, setTracks] = useRecoilState(likedTracks);
  const [playLikedTracks] = usePlayLikedTracksMutation();
  const { formatTime } = useTimeFormat();

  useEffect(() => {
    if (filter.term.length > 0 && filter.results) {
      setTracks(
        filter.results.tracks.map((x, i) => ({
          id: x.id!,
          trackNumber: i + 1,
          title: x.title,
          artist: x.artist,
          album: x.album,
          time: formatTime(x.length),
          albumArt: x.albumArt
            ? `${location.protocol}//${location.host}/covers/${x.albumArt}`
            : undefined,
          albumId: x.albumId,
          artistId: x.artistId,
          path: x.path,
        }))
      );
      return;
    }
    if (data) {
      setTracks(
        data.likedTracks.map((x, i) => ({
          id: x.id!,
          trackNumber: i + 1,
          title: x.title,
          artist: x.artist,
          album: x.album,
          time: formatTime(x.length),
          albumArt: x.albumArt
            ? `${location.protocol}//${location.host}/covers/${x.albumArt}`
            : undefined,
          albumId: x.albumId,
          artistId: x.artistId,
          path: x.path,
        }))
      );
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [filter, data]);

  useEffect(() => {
    if (!data || loading) {
      return;
    }

    const updatedLikes: Record<string, boolean> = {
      ...likes,
    };
    data.likedTracks.forEach((x) => {
      updatedLikes[x.id!] = true;
    });
    setLikes(updatedLikes);

    setTracks(
      data.likedTracks.map((x, i) => ({
        id: x.id!,
        trackNumber: i + 1,
        title: x.title,
        artist: x.artist,
        album: x.album,
        time: formatTime(x.length),
        albumArt: x.albumArt
          ? `${location.protocol}//${location.host}/covers/${x.albumArt}`
          : undefined,
        albumId: x.albumId,
        artistId: x.artistId,
        path: x.path,
      }))
    );
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [data, loading]);

  const onPlayTrack = (position: number) => {
    playLikedTracks({
      variables: {
        position,
      },
    });
  };

  const onPlayAll = () => {
    playLikedTracks();
  };

  const onShuffleAll = () => {
    playLikedTracks({
      variables: {
        shuffle: true,
      },
    });
  };

  return (
    <Likes
      tracks={tracks}
      onPlayTrack={onPlayTrack}
      onPlayAll={onPlayAll}
      onShuffleAll={onShuffleAll}
      keyword={filter.term}
      loading={loading}
    />
  );
};

export default LikesWithData;
