import { FC, ReactNode } from "react";
import { Container, Blur } from "./styles";

export type MainViewProps = {
  cover?: string;
  children?: ReactNode;
};

const MainView: FC<MainViewProps> = ({ cover, children }) => {
  return (
    <Container cover={cover}>
      {cover && <Blur>{children}</Blur>}
      {!cover && children}
    </Container>
  );
};

export default MainView;
