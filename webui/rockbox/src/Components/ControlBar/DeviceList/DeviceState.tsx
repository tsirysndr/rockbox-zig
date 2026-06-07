import { atom } from "jotai";
import { Device } from "./DeviceList";

export const deviceState = atom<{
  currentDevice: Device | null;
}>({
  currentDevice: null,
});
