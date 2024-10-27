import { FC } from "react";
import Equalizer from "./Equalizer";
import { useRecoilState } from "recoil";
import { settingsState } from "../../SettingsState";

const EqualizerWithData: FC = () => {
  const [settings] = useRecoilState(settingsState);
  console.log(">> settings", settings);

  const onEnableEq = (enable: boolean) => {
    console.log(">> enable", enable);
  };

  const onEqBandSettingsChange = (
    newEqBandSettings: {
      q: number;
      gain: number;
      cutoff: number;
    }[]
  ) => {
    console.log(">> newEqBandSettings", newEqBandSettings);
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
