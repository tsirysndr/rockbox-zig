import { atom } from "recoil";
import { CurrentTrack, Track } from "../../Types/track";

export const controlBarState = atom<{
  nowPlaying?: CurrentTrack;
  locked?: boolean;
  previousTracks?: Track[];
  nextTracks?: Track[];
}>({
  key: "controlBarState",
  default: {
    nowPlaying: undefined,
    locked: false,
    previousTracks: [],
    nextTracks: [],
  },
});
