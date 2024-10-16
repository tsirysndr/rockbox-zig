import { useEffect } from "react";
import {
  useGetCurrentPlaylistQuery,
  useGetGlobalStatusQuery,
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

  useEffect(() => {
    if (loading || !currentPlaylistData || !globalStatusData) {
      return;
    }

    if (currentPlaylistData.playlistGetCurrent.tracks.length === 0) {
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
          album: currentSong?.album,
          artist: currentSong?.artist,
          title: currentSong?.title,
          cover: currentSong?.albumArt
            ? currentSong?.albumArt.startsWith("http")
              ? currentSong.albumArt
              : `http://localhost:6062/covers/${currentSong?.albumArt}`
            : "",
          duration: currentSong?.length || 0,
          progress: globalStatusData.globalStatus.resumeElapsed,
          isPlaying: false,
          albumId: currentSong?.albumId,
        },
        resumeIndex: globalStatusData.globalStatus.resumeIndex,
      }));
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [loading, currentPlaylistData, globalStatusData]);

  return { resumePlaylistTrack };
};
