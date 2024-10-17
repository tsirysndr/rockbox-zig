import { useEffect } from "react";
import {
  useGetCurrentPlaylistQuery,
  useGetGlobalStatusQuery,
  useGetPlaybackStatusQuery,
  useResumePlaylistMutation,
  useResumePlaylistTrackMutation,
} from "./GraphQL";
import { useRecoilState } from "recoil";
import { controlBarState } from "../Components/ControlBar/ControlBarState";

export const useResumePlaylist = () => {
  const [{ resumeIndex }, setControlBarState] = useRecoilState(controlBarState);
  const { data: globalStatusData } = useGetGlobalStatusQuery();
  const {
    data: currentPlaylistData,
    loading,
    refetch: refetchCurrentPlaylist,
  } = useGetCurrentPlaylistQuery();
  const [resumePlaylist] = useResumePlaylistMutation();
  const [resumePlaylistTrack] = useResumePlaylistTrackMutation();
  const { data: getPlaybackStatusData, loading: getPlaybackStatusLoading } =
    useGetPlaybackStatusQuery();

  useEffect(() => {
    if (
      loading ||
      !currentPlaylistData ||
      !globalStatusData ||
      getPlaybackStatusLoading
    ) {
      return;
    }

    if (globalStatusData.globalStatus.resumeIndex > -1) {
      resumePlaylist()
        .then((res) => {
          if (res.data?.playlistResume === 0) {
            return refetchCurrentPlaylist();
          }
        })
        .catch((e) => console.error(e));
      return;
    }

    if (
      currentPlaylistData.playlistGetCurrent.tracks.length > 0 &&
      resumeIndex < 0
    ) {
      const currentSong =
        currentPlaylistData.playlistGetCurrent.tracks[
          globalStatusData.globalStatus.resumeIndex
        ];

      setControlBarState((state) => ({
        ...state,
        nowPlaying: {
          album:
            getPlaybackStatusData?.status === 1
              ? state.nowPlaying?.album
              : currentSong?.album,
          artist:
            getPlaybackStatusData?.status === 1
              ? state.nowPlaying?.artist
              : currentSong?.artist,
          title:
            getPlaybackStatusData?.status === 1
              ? state.nowPlaying?.title
              : currentSong?.title,
          cover:
            getPlaybackStatusData?.status === 1
              ? state.nowPlaying?.cover
              : currentSong?.albumArt
              ? currentSong?.albumArt.startsWith("http")
                ? currentSong.albumArt
                : `http://localhost:6062/covers/${currentSong?.albumArt}`
              : "",
          duration:
            getPlaybackStatusData?.status === 1
              ? state.nowPlaying?.duration || 0
              : currentSong?.length || 0,
          progress:
            getPlaybackStatusData?.status === 1
              ? state.nowPlaying?.progress || 0
              : globalStatusData.globalStatus.resumeElapsed,
          isPlaying: getPlaybackStatusData?.status === 1,
          albumId: currentSong?.albumId,
        },
        resumeIndex:
          getPlaybackStatusData?.status === 1
            ? -1
            : globalStatusData.globalStatus.resumeIndex,
      }));
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [
    loading,
    currentPlaylistData,
    globalStatusData,
    getPlaybackStatusLoading,
    getPlaybackStatusData,
  ]);

  return { resumePlaylistTrack };
};
