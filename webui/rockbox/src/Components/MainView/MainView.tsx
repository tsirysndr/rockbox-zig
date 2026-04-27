import { FC, ReactNode } from "react";
import { Container, Blur } from "./styles";

export type MainViewProps = {
  cover?: string;
  children?: ReactNode;
};

const MainView: FC<MainViewProps> = ({ cover, children }) => {
  return (
    <Container cover={cover}>
      <Blur enabled={!!cover}>{children}</Blur>
    </Container>
  );
};

export default MainView;
