import { FC } from "react";
import Sound from "./Sound";
import { settingsState } from "../SettingsState";
import { useAtom } from "jotai";
import { useSaveSettingsMutation } from "../../../Hooks/GraphQL";

const SoundWithData: FC = () => {
  const [settings, setSettings] = useAtom(settingsState);
  const { mutate: saveSettings } = useSaveSettingsMutation();

  return (
    <Sound
      bass={settings.bass}
      treble={settings.treble}
      balance={settings.balance}
      channelConfig={settings.channelConfig}
      stereoWidth={settings.stereoWidth}
      ditheringEnabled={settings.ditheringEnabled}
      afrEnabled={settings.afrEnabled}
      pbe={settings.pbe}
      pbePrecut={settings.pbePrecut}
      onBassChange={(bass) => saveSettings({ settings: { bass } })}
      onTrebleChange={(treble) => saveSettings({ settings: { treble } })}
      onBalanceChange={(balance) => saveSettings({ settings: { balance } })}
      onChannelConfigChange={(channelConfig) =>
        saveSettings({ settings: { channelConfig } })
      }
      onStereoWidthChange={(stereoWidth) =>
        saveSettings({ settings: { stereoWidth } })
      }
      onDitheringChange={(ditheringEnabled) => {
        setSettings((s) => ({ ...s, ditheringEnabled }));
        saveSettings({ settings: { ditheringEnabled } });
      }}
      onAfrChange={(afrEnabled) => {
        setSettings((s) => ({ ...s, afrEnabled }));
        saveSettings({ settings: { afrEnabled } });
      }}
      onPbeChange={(pbe, pbePrecut) => {
        setSettings((s) => ({ ...s, pbe, pbePrecut }));
        saveSettings({ settings: { pbe, pbePrecut } });
      }}
    />
  );
};

export default SoundWithData;
