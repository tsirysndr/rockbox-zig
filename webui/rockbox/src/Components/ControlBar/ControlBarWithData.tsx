import _ from "lodash";
import { FC, useEffect } from "react";
import { useRecoilState } from "recoil";
import {
  useCurrentlyPlayingSongSubscription,
  useGetCurrentTrackQuery,
  useGetGlobalSettingsQuery,
  useGetLikedAlbumsQuery,
  useGetLikedTracksQuery,
  useGetPlaybackStatusQuery,
  useLikeTrackMutation,
  useNextMutation,
  usePauseMutation,
  usePlaybackStatusSubscription,
  usePreviousMutation,
  useResumeMutation,
  useSaveSettingsMutation,
  useSeekMutation,
  useUnlikeTrackMutation,
} from "../../Hooks/GraphQL";
import { usePlayQueue } from "../../Hooks/usePlayQueue";
import { useResumePlaylist } from "../../Hooks/useResumePlaylist";
import { useSettings } from "../../Hooks/useSettings";
import { CurrentTrack } from "../../Types/track";
import { likesState } from "../Likes/LikesState";
import { settingsState } from "../Settings/SettingsState";
import ControlBar from "./ControlBar";
import { controlBarState } from "./ControlBarState";

const ControlBarWithData: FC = () => {
  const [{ nowPlaying, locked, resumeIndex }, setControlBarState] =
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
  const { resumePlaylistTrack } = useResumePlaylist();
  const [likeTrack] = useLikeTrackMutation();
  const [unlikeTrack] = useUnlikeTrackMutation();
  const [seek] = useSeekMutation();
  const [saveSettings] = useSaveSettingsMutation();
  const { refetch: refetchSettings } = useGetGlobalSettingsQuery();
  const [settings] = useRecoilState(settingsState);

  const [likes, setLikes] = useRecoilState(likesState);
  const { data: likedTracksData, loading: likedTracksLoading } =
    useGetLikedTracksQuery({
      fetchPolicy: "network-only",
    });
  const { data: likedAlbumsData, loading: likedAlbumsLoading } =
    useGetLikedAlbumsQuery({
      fetchPolicy: "network-only",
    });

  useSettings();

  useEffect(() => {
    if (
      !likedTracksData ||
      likedTracksLoading ||
      !likedAlbumsData ||
      likedAlbumsLoading
    ) {
      return;
    }

    const updatedLikes: Record<string, boolean> = {
      ...likes,
    };

    likedTracksData.likedTracks.forEach((x) => {
      updatedLikes[x.id!] = true;
    });

    likedAlbumsData.likedAlbums.forEach((x) => {
      updatedLikes[x.id!] = true;
    });

    setLikes(updatedLikes);

    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [
    likedTracksData,
    likedTracksLoading,
    likedAlbumsData,
    likedAlbumsLoading,
  ]);

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
        id: currentSong?.id || "",
        album: currentSong?.album,
        artist: currentSong?.artist,
        title: currentSong?.title,
        cover: currentSong?.albumArt
          ? currentSong?.albumArt.startsWith("http")
            ? currentSong.albumArt
            : `${location.protocol}//${location.host}/covers/${currentSong?.albumArt}`
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
      id: data.currentTrack?.id || "",
      album: data.currentTrack?.album,
      artist: data.currentTrack?.artist,
      title: data.currentTrack?.title,
      cover: data.currentTrack?.albumArt
        ? data.currentTrack?.albumArt.startsWith("http")
          ? data.currentTrack?.albumArt
          : `${location.protocol}//${location.host}/covers/${data.currentTrack?.albumArt}`
        : "",
      duration: data.currentTrack?.length || 0,
      progress: data.currentTrack?.elapsed || 0,
      isPlaying: playback?.status === 1,
      albumId: data.currentTrack?.albumId,
    });
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [data, loading, playback]);

  const onPlay = async () => {
    setControlBarState((state) => ({
      ...state,
      nowPlaying: {
        ...nowPlaying!,
        isPlaying: true,
      },
      locked: true,
    }));

    if (resumeIndex > -1) {
      try {
        await resumePlaylistTrack();
      } catch (e) {
        console.error(e);
      }

      setControlBarState((state) => ({
        ...state,
        resumeIndex: -1,
      }));

      setTimeout(() => {
        setControlBarState((state) => ({
          ...state,
          locked: false,
          resumeIndex: -1,
        }));
      }, 3000);
      return;
    }
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

  const onLike = async (trackId: string) => {
    if (!nowPlaying || !trackId) {
      return;
    }

    setLikes((state) => ({
      ...state,
      [trackId]: true,
    }));

    try {
      await likeTrack({
        variables: {
          trackId,
        },
      });
    } catch (e) {
      console.error(e);
    }
  };

  const onUnlike = async (trackId: string) => {
    if (!nowPlaying || !trackId) {
      return;
    }

    setLikes((state) => ({
      ...state,
      [trackId]: false,
    }));

    try {
      await unlikeTrack({
        variables: {
          trackId,
        },
      });
    } catch (e) {
      console.error(e);
    }
  };

  const onSeek = (elapsed: number) => {
    if (!nowPlaying) {
      return;
    }

    seek({
      variables: {
        elapsed,
        offset: 0,
      },
    });
  };

  const onShuffle = async () => {
    await saveSettings({
      variables: {
        settings: {
          playlistShuffle: !settings.playlistShuffle,
        },
      },
    });
    await refetchSettings();
  };

  const onRepeat = async () => {
    await saveSettings({
      variables: {
        settings: {
          repeatMode: settings.repeatMode === 0 ? 1 : 0,
        },
      },
    });
    await refetchSettings();
  };

  return (
    <ControlBar
      nowPlaying={nowPlaying}
      onPlay={onPlay}
      onPause={onPause}
      onNext={() => next()}
      onPrevious={() => previous()}
      onShuffle={onShuffle}
      onRepeat={onRepeat}
      shuffle={settings.playlistShuffle}
      repeat={settings.repeatMode !== 0}
      liked={likes[nowPlaying?.id || ""]}
      onLike={onLike}
      onUnlike={onUnlike}
      onSeek={onSeek}
    />
  );
};

export default ControlBarWithData;
