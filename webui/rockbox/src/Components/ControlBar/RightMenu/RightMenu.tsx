import { FC } from "react";
import { Container } from "./styles";
import { List } from "@styled-icons/entypo";
import { StatefulPopover } from "baseui/popover";
import { Button } from "../styles";
import PlayQueue from "../PlayQueue";
import { useTheme } from "@emotion/react";
import Volume from "./Volume";
import _ from "lodash";

const RightMenu: FC = () => {
  const theme = useTheme();

  return (
    <Container>
      <Volume />
      <StatefulPopover
        placement="bottom"
        content={() => <PlayQueue />}
        overrides={{
          Body: {
            style: {
              left: "-21px",
            },
          },
          Inner: {
            style: {
              backgroundColor: _.get(theme, "colors.popoverBackground", "#fff"),
            },
          },
        }}
      >
        <Button>
          <List size={21} />
        </Button>
      </StatefulPopover>
    </Container>
  );
};

export default RightMenu;
