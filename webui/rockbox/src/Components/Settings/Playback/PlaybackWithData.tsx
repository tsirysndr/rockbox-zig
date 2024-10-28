import { FC } from "react";
import Playback from "./Playback";
import { useRecoilState } from "recoil";
import { settingsState } from "../SettingsState";
import { useSaveSettingsMutation } from "../../../Hooks/GraphQL";

const PlaybackWithData: FC = () => {
  const [settings] = useRecoilState(settingsState);
  const [saveSettings] = useSaveSettingsMutation();

  const onShuffleChange = (playlistShuffle: boolean) => {
    saveSettings({
      variables: {
        settings: {
          playlistShuffle,
        },
      },
    });
  };

  const onRepeatChange = (repeatMode: number) => {
    saveSettings({
      variables: {
        settings: {
          repeatMode,
        },
      },
    });
  };

  const onCrossfadeChange = (crossfade: number) => {
    saveSettings({
      variables: {
        settings: {
          crossfade,
        },
      },
    });
  };

  const onFadeOnStopPauseChange = (fadeOnStop: boolean) => {
    saveSettings({
      variables: {
        settings: {
          fadeOnStop,
        },
      },
    });
  };

  const onReplaygainChange = (replaygain: number) => {
    saveSettings({
      variables: {
        settings: {
          replaygainSettings: {
            type: replaygain,
            preamp: settings.replaygainSettings.preamp,
            noclip: settings.replaygainSettings.noclip,
          },
        },
      },
    });
  };

  const onFadeInDelayChange = (fadeInDelay: number) => {
    saveSettings({
      variables: {
        settings: {
          fadeInDelay,
        },
      },
    });
  };

  const onFadeInDurationChange = (fadeInDuration: number) => {
    saveSettings({
      variables: {
        settings: {
          fadeInDuration,
        },
      },
    });
  };

  const onFadeOutDelayChange = (fadeOutDelay: number) => {
    saveSettings({
      variables: {
        settings: {
          fadeOutDelay,
        },
      },
    });
  };

  const onFadeOutDurationChange = (fadeOutDuration: number) => {
    saveSettings({
      variables: {
        settings: {
          fadeOutDuration,
        },
      },
    });
  };

  const onFadeOutModeChange = (fadeOutMixmode: number) => {
    saveSettings({
      variables: {
        settings: {
          fadeOutMixmode,
        },
      },
    });
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
