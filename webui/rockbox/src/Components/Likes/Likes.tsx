/* eslint-disable @typescript-eslint/no-explicit-any */
import { FC } from "react";
import Sidebar from "../Sidebar";
import ControlBar from "../ControlBar";
import { Container, ContentWrapper, Title } from "./styles";
import MainView from "../MainView";

const Likes: FC = () => {
  return (
    <Container>
      <Sidebar active="likes" />
      <MainView>
        <ControlBar />
        <ContentWrapper>
          <Title>Likes</Title>
        </ContentWrapper>
      </MainView>
    </Container>
  );
};

export default Likes;
