/* eslint-disable @typescript-eslint/no-explicit-any */
import { FC } from "react";
import Sidebar from "../Sidebar";
import MainView from "../MainView";
import ControlBar from "../ControlBar";
import Sound from "./Sound";
import Library from "./Library";
import Playback from "./Playback";

const Settings: FC = () => {
  return (
    <div className="flex flex-row w-full h-full">
      <Sidebar active="settings" />
      <MainView>
        <ControlBar />
        <div className="h-[var(--content-area-height)] overflow-y-auto">
          <div className="w-full md:w-[60vw] mx-auto mb-[100px] mt-[30px] px-4 md:px-0 md:min-w-[435px] max-w-[800px]">
            <div className="text-2xl font-[RockfordSansMedium] mx-auto mb-10">
              Settings
            </div>
            <Library />
            <Sound />
            <Playback />
          </div>
        </div>
      </MainView>
    </div>
  );
};

export default Settings;
