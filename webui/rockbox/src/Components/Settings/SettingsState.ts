import { atom } from "recoil";

export const settingsState = atom<{
  enableBlur: boolean;
}>({
  key: "settings",
  default: {
    enableBlur: false,
  },
});
