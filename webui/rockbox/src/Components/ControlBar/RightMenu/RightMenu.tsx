import { FC } from "react";
import { Container } from "./styles";
import { List } from "@styled-icons/entypo";
import { Button } from "../styles";

const RightMenu: FC = () => {
  return (
    <Container>
      <Button>
        <List size={21} />
      </Button>
    </Container>
  );
};

export default RightMenu;
