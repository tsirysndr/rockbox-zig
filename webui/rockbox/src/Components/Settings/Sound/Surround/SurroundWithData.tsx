import { FC } from "react";
import Surround from "./Surround";
import { useAtom } from "jotai";
import { settingsState } from "../../SettingsState";
import { useSaveSettingsMutation } from "../../../../Hooks/GraphQL";

const SurroundWithData: FC = () => {
  const [settings] = useAtom(settingsState);
  const { mutate: saveSettings } = useSaveSettingsMutation();

  return (
    <Surround
      enabled={settings.surroundEnabled}
      balance={settings.surroundBalance}
      fx1={settings.surroundFx1}
      fx2={settings.surroundFx2}
      mix={settings.surroundMix}
      method2={settings.surroundMethod2}
      onChange={({ enabled, balance, fx1, fx2, mix, method2 }) =>
        saveSettings({
          settings: {
            surroundEnabled: enabled,
            surroundBalance: balance,
            surroundFx1: fx1,
            surroundFx2: fx2,
            surroundMix: mix,
            surroundMethod2: method2,
          },
        })
      }
    />
  );
};

export default SurroundWithData;
