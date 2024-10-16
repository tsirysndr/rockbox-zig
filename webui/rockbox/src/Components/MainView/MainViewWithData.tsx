import { FC, ReactNode } from "react";
import MainView from "./MainView";
import { useRecoilValue } from "recoil";
import { controlBarState } from "../ControlBar/ControlBarState";
import { settingsState } from "../Settings/SettingsState";

export type MainWithDataProps = {
  children?: ReactNode;
};

const MainWithData: FC<MainWithDataProps> = ({ children }) => {
  const { nowPlaying } = useRecoilValue(controlBarState);
  const { enableBlur } = useRecoilValue(settingsState);
  return (
    <MainView cover={enableBlur ? nowPlaying?.cover : undefined}>
      {children}
    </MainView>
  );
};

export default MainWithData;
