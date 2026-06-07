import { FC } from "react";
import Sidebar from "./Sidebar";
import { controlBarState } from "../ControlBar/ControlBarState";
import { useAtomValue } from "jotai";
import { settingsState } from "../Settings/SettingsState";

const SidebarWithData: FC<{ active: string }> = (props) => {
  const { nowPlaying } = useAtomValue(controlBarState);
  const { enableBlur } = useAtomValue(settingsState);
  return (
    <Sidebar {...props} cover={enableBlur ? nowPlaying?.cover : undefined} />
  );
};

export default SidebarWithData;
