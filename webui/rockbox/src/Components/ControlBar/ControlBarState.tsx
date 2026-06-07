import { atom } from "jotai";
import { CurrentTrack, Track } from "../../Types/track";

export const controlBarState = atom<{
  nowPlaying?: CurrentTrack;
  locked?: boolean;
  previousTracks?: Track[];
  nextTracks?: Track[];
  resumeIndex: number;
}>({
  nowPlaying: undefined,
  locked: false,
  previousTracks: [],
  nextTracks: [],
  resumeIndex: -1,
});
