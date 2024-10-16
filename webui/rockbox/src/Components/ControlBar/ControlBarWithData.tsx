import { FC, useEffect } from "react";
import ControlBar from "./ControlBar";
import {
  useCurrentlyPlayingSongSubscription,
  useGetCurrentTrackQuery,
  useGetPlaybackStatusQuery,
  useNextMutation,
  usePauseMutation,
  usePlaybackStatusSubscription,
  usePreviousMutation,
  useResumeMutation,
} from "../../Hooks/GraphQL";
import { CurrentTrack } from "../../Types/track";
import _ from "lodash";
import { useRecoilState } from "recoil";
import { controlBarState } from "./ControlBarState";
import { usePlayQueue } from "../../Hooks/usePlayQueue";

const ControlBarWithData: FC = () => {
  const [{ nowPlaying, locked }, setControlBarState] =
    useRecoilState(controlBarState);
  const { data, loading } = useGetCurrentTrackQuery();
  const { data: playback } = useGetPlaybackStatusQuery({
    fetchPolicy: "network-only",
  });
  const [pause] = usePauseMutation();
  const [resume] = useResumeMutation();
  const [previous] = usePreviousMutation();
  const [next] = useNextMutation();
  const { data: playbackSubscription } = useCurrentlyPlayingSongSubscription();
  const { data: playbackStatus } = usePlaybackStatusSubscription();
  const { previousTracks, nextTracks } = usePlayQueue();

  const setNowPlaying = (nowPlaying: CurrentTrack) => {
    setControlBarState((state) => ({
      ...state,
      nowPlaying,
    }));
  };

  useEffect(() => {
    setControlBarState((state) => ({
      ...state,
      nextTracks,
      previousTracks,
    }));
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [nextTracks, previousTracks]);

  useEffect(() => {
    if (_.get(playbackSubscription, "currentlyPlayingSong.length", 0) > 0) {
      const currentSong = playbackSubscription?.currentlyPlayingSong;
      setNowPlaying({
        album: currentSong?.album,
        artist: currentSong?.artist,
        title: currentSong?.title,
        cover: currentSong?.albumArt
          ? currentSong?.albumArt.startsWith("http")
            ? currentSong.albumArt
            : `http://localhost:6062/covers/${currentSong?.albumArt}`
          : "",
        duration: currentSong?.length || 0,
        progress: currentSong?.elapsed || 0,
        isPlaying: !locked
          ? playbackStatus?.playbackStatus.status === 1
          : nowPlaying?.isPlaying,
        albumId: currentSong?.albumId,
      });
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [playbackSubscription, playbackStatus]);

  useEffect(() => {
    if (loading || !data) {
      return;
    }

    if (nowPlaying) {
      return;
    }

    setNowPlaying({
      album: data.currentTrack?.album,
      artist: data.currentTrack?.artist,
      title: data.currentTrack?.title,
      cover: data.currentTrack?.albumArt
        ? data.currentTrack?.albumArt.startsWith("http")
          ? data.currentTrack?.albumArt
          : `http://localhost:6062/covers/${data.currentTrack?.albumArt}`
        : "",
      duration: data.currentTrack?.length || 0,
      progress: data.currentTrack?.elapsed || 0,
      isPlaying: playback?.status === 1,
      albumId: data.currentTrack?.albumId,
    });
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [data, loading, playback]);

  const onPlay = () => {
    setControlBarState((state) => ({
      ...state,
      nowPlaying: {
        ...nowPlaying!,
        isPlaying: true,
      },
      locked: true,
    }));
    resume();
    setTimeout(() => {
      setControlBarState((state) => ({
        ...state,
        locked: false,
      }));
    }, 3000);
  };

  const onPause = () => {
    setControlBarState((state) => ({
      ...state,
      nowPlaying: {
        ...nowPlaying!,
        isPlaying: false,
      },
      locked: true,
    }));
    pause();
    setTimeout(() => {
      setControlBarState((state) => ({
        ...state,
        locked: true,
      }));
    }, 3000);
  };

  return (
    <ControlBar
      nowPlaying={nowPlaying}
      onPlay={onPlay}
      onPause={onPause}
      onNext={() => next()}
      onPrevious={() => previous()}
      onShuffle={() => {}}
      onRepeat={() => {}}
    />
  );
};

export default ControlBarWithData;
