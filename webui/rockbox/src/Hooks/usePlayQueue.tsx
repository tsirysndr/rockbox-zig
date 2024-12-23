import { useMemo } from "react";
import {
  useGetCurrentPlaylistQuery,
  usePlaylistChangedSubscription,
} from "./GraphQL";
import _ from "lodash";
import { useRecoilValue } from "recoil";
import { controlBarState } from "../Components/ControlBar/ControlBarState";
import { deviceState } from "../Components/ControlBar/DeviceList/DeviceState";

export const usePlayQueue = () => {
  const { currentDevice } = useRecoilValue(deviceState);
  const { resumeIndex } = useRecoilValue(controlBarState);
  const { data: playlistSubscription } = usePlaylistChangedSubscription({
    fetchPolicy: "network-only",
  });
  const { data } = useGetCurrentPlaylistQuery({
    fetchPolicy: "cache-and-network",
  });
  const previousTracks = useMemo(() => {
    if (playlistSubscription?.playlistChanged) {
      const currentTrackIndex =
        resumeIndex > -1 && currentDevice === null
          ? resumeIndex
          : _.get(playlistSubscription, "playlistChanged.index", 0);
      const tracks = _.get(playlistSubscription, "playlistChanged.tracks", []);
      return tracks.slice(0, currentTrackIndex + 1).map((x, index) => ({
        ...x,
        id: index.toString(),
        cover: x.albumArt
          ? x.albumArt.startsWith("http")
            ? x.albumArt
            : `http://localhost:6062/covers/${x.albumArt}`
          : undefined,
      }));
    }
    const currentTrackIndex =
      resumeIndex > -1
        ? resumeIndex
        : _.get(data, "playlistGetCurrent.index", 0);
    const tracks = _.get(data, "playlistGetCurrent.tracks", []);
    return tracks.slice(0, currentTrackIndex + 1).map((x, index) => ({
      ...x,
      id: index.toString(),
      cover: x.albumArt
        ? x.albumArt.startsWith("http")
          ? x.albumArt
          : `http://localhost:6062/covers/${x.albumArt}`
        : undefined,
    }));
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [data, playlistSubscription, resumeIndex]);

  const nextTracks = useMemo(() => {
    if (playlistSubscription?.playlistChanged) {
      const currentTrackIndex =
        resumeIndex > -1
          ? resumeIndex
          : _.get(playlistSubscription, "playlistChanged.index", 0);
      const tracks = _.get(playlistSubscription, "playlistChanged.tracks", []);
      return tracks.slice(currentTrackIndex + 1).map((x, index) => ({
        ...x,
        id: index.toString(),
        cover: x.albumArt
          ? x.albumArt.startsWith("http")
            ? x.albumArt
            : `http://localhost:6062/covers/${x.albumArt}`
          : undefined,
      }));
    }
    const currentTrackIndex =
      resumeIndex > -1
        ? resumeIndex
        : _.get(data, "playlistGetCurrent.index", 0);
    const tracks = _.get(data, "playlistGetCurrent.tracks", []);
    return tracks.slice(currentTrackIndex + 1).map((x, index) => ({
      ...x,
      id: index.toString(),
      cover: x.albumArt
        ? x.albumArt.startsWith("http")
          ? x.albumArt
          : `http://localhost:6062/covers/${x.albumArt}`
        : undefined,
    }));
  }, [data, playlistSubscription, resumeIndex]);

  return { previousTracks, nextTracks };
};
