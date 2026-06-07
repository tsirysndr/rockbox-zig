import { atom } from "jotai";
import { Track } from "../../Types/track";

export const likesState = atom<Record<string, boolean>>({});

export const likedTracks = atom<Track[]>([]);
