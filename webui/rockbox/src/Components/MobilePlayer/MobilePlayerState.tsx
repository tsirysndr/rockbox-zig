import { atom } from "jotai";

export const mobilePlayerState = atom<{ isOpen: boolean }>({ isOpen: false });
