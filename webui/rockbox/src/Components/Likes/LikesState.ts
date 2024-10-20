import { atom } from "recoil";
import { Track } from "../../Types/track";

export const likesState = atom<Record<string, boolean>>({
  key: "likes",
  default: {},
});

export const likedTracks = atom<Track[]>({
  key: "likedTracks",
  default: [],
});
