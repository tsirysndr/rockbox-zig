import { FC } from "react";
import Sidebar from "../Sidebar";
import ControlBar from "../ControlBar";
import { Container, MainView } from "./styles";

const Tracks: FC = () => {
  return (
    <Container>
      <Sidebar active="songs" />
      <MainView>
        <ControlBar />
      </MainView>
    </Container>
  );
};

export default Tracks;
