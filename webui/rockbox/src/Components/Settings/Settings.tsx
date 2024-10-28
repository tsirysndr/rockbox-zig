/* eslint-disable @typescript-eslint/no-explicit-any */
import { FC } from "react";
import Sidebar from "../Sidebar";
import MainView from "../MainView";
import ControlBar from "../ControlBar";
import { Container, Scrollable, Title, Wrapper } from "./styles";
import Sound from "./Sound";
import Library from "./Library";
import Playback from "./Playback";

const Settings: FC = () => {
  return (
    <Container>
      <Sidebar active="settings" />
      <MainView>
        <ControlBar />
        <Scrollable>
          <Wrapper>
            <Title>Settings</Title>
            <Library />
            <Sound />
            <Playback />
          </Wrapper>
        </Scrollable>
      </MainView>
    </Container>
  );
};

export default Settings;
