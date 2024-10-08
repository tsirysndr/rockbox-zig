import { FC } from "react";
import Sidebar from "../Sidebar";
import ControlBar from "../ControlBar";
import { Container, MainView, Title } from "./styles";

const Files: FC = () => {
  return (
    <Container>
      <Sidebar active="files" />
      <MainView>
        <ControlBar />
        <Title>Files</Title>
      </MainView>
    </Container>
  );
};

export default Files;
