import { FC } from "react";
import { Container } from "./styles";
import { List } from "@styled-icons/entypo";
import { StatefulPopover } from "baseui/popover";
import { Button } from "../styles";
import PlayQueue from "../PlayQueue";
import { useTheme } from "@emotion/react";
import Volume from "./Volume";
import _ from "lodash";
import { Speaker } from "@styled-icons/bootstrap";
import DeviceList from "../DeviceList";

const RightMenu: FC = () => {
  const theme = useTheme();

  return (
    <Container>
      <Volume />
      <StatefulPopover
        placement="bottom"
        content={({ close }) => <DeviceList close={close} />}
        overrides={{
          Body: {
            style: {
              top: "10px",
              left: "-70px",
            },
          },
          Inner: {
            style: {
              backgroundColor: theme.colors.popoverBackground,
            },
          },
        }}
      >
        <button
          style={{
            border: "none",
            backgroundColor: "initial",
            cursor: "pointer",
          }}
        >
          <Speaker size={18} color={theme.colors.icon} />
        </button>
      </StatefulPopover>

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
