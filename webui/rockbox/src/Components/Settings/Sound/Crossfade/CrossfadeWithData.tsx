import { FC } from "react";
import Crossfade from "./Crossfade";
import { useAtom } from "jotai";
import { settingsState } from "../../SettingsState";
import { useSaveSettingsMutation } from "../../../../Hooks/GraphQL";

const CrossfadeWithData: FC = () => {
  const [settings] = useAtom(settingsState);
  const { mutate: saveSettings } = useSaveSettingsMutation();

  return (
    <Crossfade
      mode={settings.crossfade}
      fadeOnStop={settings.fadeOnStop}
      fadeInDelay={settings.crossfadeFadeInDelay}
      fadeOutDelay={settings.crossfadeFadeOutDelay}
      fadeInDuration={settings.crossfadeFadeInDuration}
      fadeOutDuration={settings.crossfadeFadeOutDuration}
      mixmode={settings.crossfadeFadeOutMixmode}
      onChange={({ mode, fadeOnStop, fadeInDelay, fadeOutDelay, fadeInDuration, fadeOutDuration, mixmode }) =>
        saveSettings({
          settings: {
            crossfade: mode,
            fadeOnStop,
            fadeInDelay,
            fadeOutDelay,
            fadeInDuration,
            fadeOutDuration,
            fadeOutMixmode: mixmode,
          },
        })
      }
    />
  );
};

export default CrossfadeWithData;
