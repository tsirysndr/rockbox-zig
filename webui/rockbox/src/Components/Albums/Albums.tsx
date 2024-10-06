import { FC } from "react";
import Sidebar from "../Sidebar";
import ControlBar from "../ControlBar";
import { Container, MainView } from "./styles";

const Albums: FC = () => {
  return (
    <Container>
      <Sidebar active="albums" />
      <MainView>
        <ControlBar />
      </MainView>
    </Container>
  );
};

export default Albums;
