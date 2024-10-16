import { useMemo } from "react";
import {
  useGetCurrentPlaylistQuery,
  usePlaylistChangedSubscription,
} from "./GraphQL";
import _ from "lodash";

export const usePlayQueue = () => {
  const { data: playlistSubscription } = usePlaylistChangedSubscription({
    fetchPolicy: "network-only",
  });
  const { data } = useGetCurrentPlaylistQuery({
    fetchPolicy: "cache-and-network",
  });
  const previousTracks = useMemo(() => {
    if (playlistSubscription?.playlistChanged) {
      const currentTrackIndex = _.get(
        playlistSubscription,
        "playlistChanged.index",
        0
      );
      const tracks = _.get(playlistSubscription, "playlistChanged.tracks", []);
      return tracks.slice(0, currentTrackIndex + 1).map((x, index) => ({
        ...x,
        id: index.toString(),
      }));
    }
    const currentTrackIndex = _.get(data, "playlistGetCurrent.index", 0);
    const tracks = _.get(data, "playlistGetCurrent.tracks", []);
    return tracks.slice(0, currentTrackIndex + 1).map((x, index) => ({
      ...x,
      id: index.toString(),
    }));
  }, [data, playlistSubscription]);

  const nextTracks = useMemo(() => {
    if (playlistSubscription?.playlistChanged) {
      const currentTrackIndex = _.get(
        playlistSubscription,
        "playlistChanged.index",
        0
      );
      const tracks = _.get(playlistSubscription, "playlistChanged.tracks", []);
      return tracks.slice(currentTrackIndex + 1).map((x, index) => ({
        ...x,
        id: index.toString(),
      }));
    }
    const currentTrackIndex = _.get(data, "playlistGetCurrent.index", 0);
    const tracks = _.get(data, "playlistGetCurrent.tracks", []);
    return tracks.slice(currentTrackIndex + 1).map((x, index) => ({
      ...x,
      id: index.toString(),
    }));
  }, [data, playlistSubscription]);

  return { previousTracks, nextTracks };
};
