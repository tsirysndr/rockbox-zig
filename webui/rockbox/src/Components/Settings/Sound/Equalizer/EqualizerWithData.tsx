import { FC } from "react";
import Equalizer from "./Equalizer";
import { useRecoilState } from "recoil";
import { settingsState } from "../../SettingsState";
import { useSaveSettingsMutation } from "../../../../Hooks/GraphQL";

const EqualizerWithData: FC = () => {
  const [settings] = useRecoilState(settingsState);
  const [saveSettings] = useSaveSettingsMutation();

  const onEnableEq = (eqEnabled: boolean) => {
    saveSettings({
      variables: {
        settings: {
          eqEnabled,
        },
      },
    });
  };

  const onEqBandSettingsChange = (
    eqBandSettings: {
      q: number;
      gain: number;
      cutoff: number;
    }[]
  ) => {
    saveSettings({
      variables: {
        settings: {
          eqBandSettings,
        },
      },
    });
  };

  return (
    <Equalizer
      eqEnabled={settings.eqEnabled}
      onEnableEq={onEnableEq}
      eqBandSettings={settings.eqBandSettings}
      onEqBandSettingsChange={onEqBandSettingsChange}
    />
  );
};

export default EqualizerWithData;
