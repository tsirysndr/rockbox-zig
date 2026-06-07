// Mount once at the app root. Watches the current device and, whenever the
// active output switches to the CMAF (HLS) sink, attaches the browser's HLS
// player to the daemon's stream. Detaches only when the user explicitly
// switches to a different *known* output — we never detach on a transient
// undefined `currentDevice`, since route navigation can briefly null out the
// upstream query result and we don't want playback to stutter.
//
// The stream URL is built from the page hostname + the `cmafHttpPort`
// returned by the globalSettings GraphQL query, falling back to 7882 (the
// rockboxd default) when the setting is null.

import { FC, useEffect } from "react";
import { useRecoilValue } from "recoil";

import { useGetGlobalSettingsQuery } from "../../Hooks/GraphQL";
import { hlsAudio } from "../../lib/hls-audio";
import { deviceState } from "../ControlBar/DeviceList/DeviceState";

const DEFAULT_CMAF_HTTP_PORT = 7882;

function isCmafDevice(type?: string | null): boolean {
  if (!type) return false;
  const t = type.toLowerCase();
  return t === "cmaf" || t === "hls" || t === "dash";
}

const HlsAutoConnect: FC = () => {
  const { currentDevice } = useRecoilValue(deviceState);
  const { data: settingsData } = useGetGlobalSettingsQuery();
  const cmafPort =
    settingsData?.globalSettings?.cmafHttpPort ?? DEFAULT_CMAF_HTTP_PORT;

  useEffect(() => {
    // No device info yet — leave whatever is playing alone. Detaching on
    // every transient `undefined` is what made the player stop mid-track
    // when navigating between screens.
    if (!currentDevice) return;
    if (isCmafDevice(currentDevice.type)) {
      const url = `${window.location.protocol}//${window.location.hostname}:${cmafPort}/hls/master.m3u8`;
      hlsAudio.attach(url);
    } else {
      hlsAudio.detach();
    }
  }, [currentDevice, cmafPort]);

  return null;
};

export default HlsAutoConnect;
