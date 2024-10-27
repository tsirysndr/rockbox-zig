import { FC } from "react";
import Sound from "./Sound";
import { settingsState } from "../SettingsState";
import { useRecoilState } from "recoil";
import { useSaveSettingsMutation } from "../../../Hooks/GraphQL";

const SoundWithData: FC = () => {
  const [settings] = useRecoilState(settingsState);
  const [saveSettings] = useSaveSettingsMutation();

  const onBalanceChange = (balance: number) => {
    saveSettings({
      variables: {
        settings: {
          balance,
        },
      },
    });
  };

  const onBassChange = (bass: number) => {
    console.log(">> bass", bass);
  };
  const onTrebleChange = (treble: number) => {
    console.log(">> treble", treble);
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
