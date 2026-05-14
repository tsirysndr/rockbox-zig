import { FC } from "react";
import Crossfeed from "./Crossfeed";
import { useRecoilState } from "recoil";
import { settingsState } from "../../SettingsState";
import { useSaveSettingsMutation } from "../../../../Hooks/GraphQL";

const CrossfeedWithData: FC = () => {
  const [settings] = useRecoilState(settingsState);
  const { mutate: saveSettings } = useSaveSettingsMutation();

  return (
    <Crossfeed
      type={settings.crossfeedType}
      directGain={settings.crossfeedDirectGain}
      crossGain={settings.crossfeedCrossGain}
      hfAttenuation={settings.crossfeedHfAttenuation}
      hfCutoff={settings.crossfeedHfCutoff}
      onChange={({ type, directGain, crossGain, hfAttenuation, hfCutoff }) =>
        saveSettings({
          settings: {
            crossfeed: type,
            crossfeedDirectGain: directGain,
            crossfeedCrossGain: crossGain,
            crossfeedHfAttenuation: hfAttenuation,
            crossfeedHfCutoff: hfCutoff,
          },
        })
      }
    />
  );
};

export default CrossfeedWithData;
