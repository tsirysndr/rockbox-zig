import { FC } from "react";
import ReplayGain from "./ReplayGain";
import { useRecoilState } from "recoil";
import { settingsState } from "../../SettingsState";
import { useSaveSettingsMutation } from "../../../../Hooks/GraphQL";

const ReplayGainWithData: FC = () => {
  const [settings] = useRecoilState(settingsState);
  const { mutate: saveSettings } = useSaveSettingsMutation();

  return (
    <ReplayGain
      type={settings.replaygainSettings.type}
      preamp={settings.replaygainSettings.preamp}
      noclip={settings.replaygainSettings.noclip}
      onChange={({ type, preamp, noclip }) =>
        saveSettings({
          settings: {
            replaygainSettings: { type, preamp, noclip },
          },
        })
      }
    />
  );
};

export default ReplayGainWithData;
