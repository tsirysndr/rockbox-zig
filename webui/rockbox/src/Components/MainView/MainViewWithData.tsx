import { FC, ReactNode } from "react";
import MainView from "./MainView";
import { useAtomValue } from "jotai";
import { controlBarState } from "../ControlBar/ControlBarState";
import { settingsState } from "../Settings/SettingsState";

export type MainWithDataProps = {
  children?: ReactNode;
};

const MainWithData: FC<MainWithDataProps> = ({ children }) => {
  const { nowPlaying } = useAtomValue(controlBarState);
  const { enableBlur } = useAtomValue(settingsState);
  return (
    <MainView cover={enableBlur ? nowPlaying?.cover : undefined}>
      {children}
    </MainView>
  );
};

export default MainWithData;
