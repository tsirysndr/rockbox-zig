import { FC } from "react";
import Playback from "./Playback";
import { useRecoilState } from "recoil";
import { settingsState } from "../SettingsState";

const PlaybackWithData: FC = () => {
  const [settings] = useRecoilState(settingsState);
  const onShuffleChange = (shuffle: boolean) => {
    console.log(">> shuffle", shuffle);
  };
  const onRepeatChange = (repeat: number) => {
    console.log(">> repeat", repeat);
  };
  const onCrossfadeChange = (crossfade: number) => {
    console.log(">> crossfade", crossfade);
  };
  const onFadeOnStopPauseChange = (fadeOnStopPause: boolean) => {
    console.log(">> fadeOnStopPause", fadeOnStopPause);
  };
  const onReplaygainChange = (replaygain: number) => {
    console.log(">> replaygain", replaygain);
  };
  const onFadeInDelayChange = (fadeInDelay: number) => {
    console.log(">> fadeInDelay", fadeInDelay);
  };
  const onFadeInDurationChange = (fadeInDuration: number) => {
    console.log(">> fadeInDuration", fadeInDuration);
  };
  const onFadeOutDelayChange = (fadeOutDelay: number) => {
    console.log(">> fadeOutDelay", fadeOutDelay);
  };
  const onFadeOutDurationChange = (fadeOutDuration: number) => {
    console.log(">> fadeOutDuration", fadeOutDuration);
  };
  const onFadeOutModeChange = (fadeOutMode: number) => {
    console.log(">> fadeOutMode", fadeOutMode);
  };

  return (
    <Playback
      shuffle={settings.playlistShuffle}
      repeat={settings.repeatMode}
      fadeOnStopPause={settings.fadeOnStop}
      crossfade={settings.crossfade}
      replaygain={settings.replaygainSettings.type}
      fadeInDelay={settings.crossfadeFadeInDelay}
      fadeInDuration={settings.crossfadeFadeInDuration}
      fadeOutDelay={settings.crossfadeFadeOutDelay}
      fadeOutDuration={settings.crossfadeFadeOutDuration}
      fadeOutMode={settings.crossfadeFadeOutMixmode}
      onShuffleChange={onShuffleChange}
      onRepeatChange={onRepeatChange}
      onCrossfadeChange={onCrossfadeChange}
      onFadeOnStopPauseChange={onFadeOnStopPauseChange}
      onReplaygainChange={onReplaygainChange}
      onFadeInDelayChange={onFadeInDelayChange}
      onFadeInDurationChange={onFadeInDurationChange}
      onFadeOutDelayChange={onFadeOutDelayChange}
      onFadeOutDurationChange={onFadeOutDurationChange}
      onFadeOutModeChange={onFadeOutModeChange}
    />
  );
};

export default PlaybackWithData;
