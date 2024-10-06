import { FC } from "react";
import Sidebar from "../Sidebar";
import ControlBar from "../ControlBar";
import { Container, MainView } from "./styles";

const Artists: FC = () => {
  return (
    <Container>
      <Sidebar active="artists" />
      <MainView>
        <ControlBar />
      </MainView>
    </Container>
  );
};

export default Artists;
