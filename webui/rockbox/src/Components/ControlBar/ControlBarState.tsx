import { atom } from "recoil";
import { CurrentTrack } from "../../Types/track";

export const controlBarState = atom<{
  nowPlaying?: CurrentTrack;
  locked?: boolean;
}>({
  key: "controlBarState",
  default: {
    nowPlaying: undefined,
    locked: false,
  },
});
