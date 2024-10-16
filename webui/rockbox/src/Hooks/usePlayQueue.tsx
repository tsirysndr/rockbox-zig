import { useMemo } from "react";
import {
  useGetCurrentPlaylistQuery,
  usePlaylistChangedSubscription,
} from "./GraphQL";
import _ from "lodash";
import { useRecoilValue } from "recoil";
import { controlBarState } from "../Components/ControlBar/ControlBarState";

export const usePlayQueue = () => {
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
        resumeIndex > -1
          ? resumeIndex
          : _.get(playlistSubscription, "playlistChanged.index", 0);
      const tracks = _.get(playlistSubscription, "playlistChanged.tracks", []);
      return tracks.slice(0, currentTrackIndex + 1).map((x, index) => ({
        ...x,
        id: index.toString(),
        cover: x.albumArt
          ? `http://localhost:6062/covers/${x.albumArt}`
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
        ? `http://localhost:6062/covers/${x.albumArt}`
        : undefined,
    }));
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
          ? `http://localhost:6062/covers/${x.albumArt}`
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
        ? `http://localhost:6062/covers/${x.albumArt}`
        : undefined,
    }));
  }, [data, playlistSubscription, resumeIndex]);

  return { previousTracks, nextTracks };
};
