import { FC } from "react";
import Playback from "./Playback";
import { useRecoilState } from "recoil";
import { settingsState } from "../SettingsState";
import {
  useGetGlobalSettingsQuery,
  useSaveSettingsMutation,
} from "../../../Hooks/GraphQL";

const PlaybackWithData: FC = () => {
  const { refetch: refetchSettings } = useGetGlobalSettingsQuery();
  const [settings] = useRecoilState(settingsState);
  const { mutate: saveSettings, mutateAsync: saveSettingsAsync } = useSaveSettingsMutation();

  const onShuffleChange = async (playlistShuffle: boolean) => {
    await saveSettingsAsync({ settings: { playlistShuffle } });
    await refetchSettings();
  };

  const onRepeatChange = async (repeatMode: number) => {
    await saveSettingsAsync({ settings: { repeatMode } });
    await refetchSettings();
  };

  const onCrossfadeChange = (crossfade: number) => {
    saveSettings({ settings: { crossfade } });
  };

  const onFadeOnStopPauseChange = (fadeOnStop: boolean) => {
    saveSettings({ settings: { fadeOnStop } });
  };

  const onReplaygainChange = (replaygain: number) => {
    saveSettings({
      settings: {
        replaygainSettings: {
          type: replaygain,
          preamp: settings.replaygainSettings.preamp,
          noclip: settings.replaygainSettings.noclip,
        },
      },
    });
  };

  const onFadeInDelayChange = (fadeInDelay: number) => {
    saveSettings({ settings: { fadeInDelay } });
  };

  const onFadeInDurationChange = (fadeInDuration: number) => {
    saveSettings({ settings: { fadeInDuration } });
  };

  const onFadeOutDelayChange = (fadeOutDelay: number) => {
    saveSettings({ settings: { fadeOutDelay } });
  };

  const onFadeOutDurationChange = (fadeOutDuration: number) => {
    saveSettings({ settings: { fadeOutDuration } });
  };

  const onFadeOutModeChange = (fadeOutMixmode: number) => {
    saveSettings({ settings: { fadeOutMixmode } });
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
