import { FC } from "react";
import Sound from "./Sound";
import { settingsState } from "../SettingsState";
import { useRecoilState } from "recoil";
import { useSaveSettingsMutation } from "../../../Hooks/GraphQL";

const SoundWithData: FC = () => {
  const [settings] = useRecoilState(settingsState);
  const { mutate: saveSettings } = useSaveSettingsMutation();

  const onBalanceChange = (balance: number) => {
    saveSettings({ settings: { balance } });
  };

  const onBassChange = (bass: number) => {
    saveSettings({ settings: { bass } });
  };

  const onTrebleChange = (treble: number) => {
    saveSettings({ settings: { treble } });
  };
  return (
    <Sound
      bass={settings.bass}
      treble={settings.treble}
      balance={settings.balance}
      onBalanceChange={onBalanceChange}
      onBassChange={onBassChange}
      onTrebleChange={onTrebleChange}
    />
  );
};

export default SoundWithData;
