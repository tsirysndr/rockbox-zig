import { FC } from "react";
import Settings from "./Settings";
import { useSettings } from "../../Hooks/useSettings";

const SettingsWithData: FC = () => {
  useSettings();
  return <Settings />;
};

export default SettingsWithData;
