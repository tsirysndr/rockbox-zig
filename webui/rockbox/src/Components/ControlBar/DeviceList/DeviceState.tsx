import { atom } from "recoil";
import { Device } from "./DeviceList";

export const deviceState = atom<{
  currentDevice: Device | null;
}>({
  key: "deviceState",
  default: {
    currentDevice: null,
  },
});
