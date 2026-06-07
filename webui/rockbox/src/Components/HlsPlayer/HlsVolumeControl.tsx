// Local volume + mute control for the browser-side HLS player.
//
// Only renders when the user is connected to the CMAF sink — when the
// rockboxd output is anything else, the audio they hear comes from elsewhere
// (the device's own speakers, an AirPlay receiver, etc.) and this slider
// would have no effect, so we hide it to avoid confusion.

import { FC } from "react";
import { useRecoilValue } from "recoil";
import {
  VolumeHigh,
  VolumeMute,
  VolumeLow,
} from "@styled-icons/ionicons-outline";

import { hlsAudio, useHlsAudio } from "../../lib/hls-audio";
import { deviceState } from "../ControlBar/DeviceList/DeviceState";

function isCmafDevice(type?: string | null): boolean {
  if (!type) return false;
  const t = type.toLowerCase();
  return t === "cmaf" || t === "hls" || t === "dash";
}

const HlsVolumeControl: FC = () => {
  const { currentDevice } = useRecoilValue(deviceState);
  const { attached, volume, muted } = useHlsAudio();

  if (!currentDevice || !isCmafDevice(currentDevice.type) || !attached) {
    return null;
  }

  const Icon = muted ? VolumeMute : volume < 0.5 ? VolumeLow : VolumeHigh;

  return (
    <div
      className="flex items-center gap-2 mr-3"
      title="Volume (browser HLS player)"
    >
      <button
        type="button"
        onClick={() => hlsAudio.toggleMute()}
        className="border-0 bg-transparent p-0 cursor-pointer leading-none"
        aria-label={muted ? "Unmute" : "Mute"}
      >
        <Icon size={18} color={muted ? "#ff5a5a" : "var(--theme-icon)"} />
      </button>
      <input
        type="range"
        min={0}
        max={1}
        step={0.01}
        value={muted ? 0 : volume}
        onChange={(e) => hlsAudio.setVolume(Number(e.target.value))}
        aria-label="HLS volume"
        className={[
          // Track: thin pill.
          "w-24 h-1 cursor-pointer appearance-none rounded-full bg-white/20",
          // Use the theme accent for both the filled track and the thumb.
          "accent-[var(--theme-text)]",
          // WebKit/Blink — flatten the runnable-track and shrink the thumb.
          "[&::-webkit-slider-runnable-track]:h-1",
          "[&::-webkit-slider-runnable-track]:rounded-full",
          "[&::-webkit-slider-runnable-track]:bg-transparent",
          "[&::-webkit-slider-thumb]:appearance-none",
          "[&::-webkit-slider-thumb]:h-3",
          "[&::-webkit-slider-thumb]:w-3",
          "[&::-webkit-slider-thumb]:rounded-full",
          "[&::-webkit-slider-thumb]:bg-[var(--theme-text)]",
          "[&::-webkit-slider-thumb]:-mt-1",
          "[&::-webkit-slider-thumb]:cursor-pointer",
          // Firefox.
          "[&::-moz-range-track]:h-1",
          "[&::-moz-range-track]:rounded-full",
          "[&::-moz-range-track]:bg-white/20",
          "[&::-moz-range-thumb]:h-3",
          "[&::-moz-range-thumb]:w-3",
          "[&::-moz-range-thumb]:rounded-full",
          "[&::-moz-range-thumb]:border-0",
          "[&::-moz-range-thumb]:bg-[var(--theme-text)]",
          "[&::-moz-range-thumb]:cursor-pointer",
        ].join(" ")}
      />
    </div>
  );
};

export default HlsVolumeControl;
