import { atom } from "recoil";

export const mobilePlayerState = atom<{ isOpen: boolean }>({
  key: "mobilePlayerState",
  default: { isOpen: false },
});
