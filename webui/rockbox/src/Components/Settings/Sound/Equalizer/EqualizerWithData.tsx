import { FC } from "react";
import Equalizer from "./Equalizer";
import { useAtom } from "jotai";
import { settingsState } from "../../SettingsState";
import { useSaveSettingsMutation } from "../../../../Hooks/GraphQL";

const EqualizerWithData: FC = () => {
  const [settings] = useAtom(settingsState);
  const { mutate: saveSettings } = useSaveSettingsMutation();

  const onEnableEq = (eqEnabled: boolean) => {
    // Send band settings alongside the enable flag so the firmware always has
    // the correct WASM preset cutoff/q values (not the firmware defaults).
    saveSettings({ settings: { eqEnabled, eqBandSettings: settings.eqBandSettings } });
  };

  const onEqPrecutChange = (eqPrecut: number) => {
    saveSettings({ settings: { eqPrecut } });
  };

  const onEqBandSettingsChange = (
    eqBandSettings: {
      q: number;
      gain: number;
      cutoff: number;
    }[]
  ) => {
    saveSettings({ settings: { eqBandSettings } });
  };

  return (
    <Equalizer
      eqEnabled={settings.eqEnabled}
      eqPrecut={settings.eqPrecut}
      onEnableEq={onEnableEq}
      onEqPrecutChange={onEqPrecutChange}
      eqBandSettings={settings.eqBandSettings}
      onEqBandSettingsChange={onEqBandSettingsChange}
    />
  );
};

export default EqualizerWithData;
