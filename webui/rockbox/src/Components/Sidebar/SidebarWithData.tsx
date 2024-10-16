import { FC } from "react";
import Sidebar from "./Sidebar";
import { controlBarState } from "../ControlBar/ControlBarState";
import { useRecoilValue } from "recoil";
import { settingsState } from "../Settings/SettingsState";

const SidebarWithData: FC<{ active: string }> = (props) => {
  const { nowPlaying } = useRecoilValue(controlBarState);
  const { enableBlur } = useRecoilValue(settingsState);
  return (
    <Sidebar {...props} cover={enableBlur ? nowPlaying?.cover : undefined} />
  );
};

export default SidebarWithData;
