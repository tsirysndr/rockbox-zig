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
import BluetoothList from "../BluetoothList";
import { useGetBluetoothDevicesQuery } from "../../../Hooks/GraphQL";

const BluetoothIcon: FC<{ color: string }> = ({ color }) => (
  <svg viewBox="0 0 24 24" width={18} height={18} fill={color}>
    <path d="M17.71 7.71L12 2h-1v7.59L6.41 5 5 6.41 10.59 12 5 17.59 6.41 19 11 14.41V22h1l5.71-5.71-4.3-4.29 4.3-4.29zM13 5.83l1.88 1.88L13 9.59V5.83zm1.88 10.46L13 18.17v-3.76l1.88 1.88z" />
  </svg>
);

const RightMenu: FC = () => {
  const theme = useTheme();
  const { isError: bluetoothUnavailable } = useGetBluetoothDevicesQuery({
    retry: false,
  });
  const bluetoothAvailable = !bluetoothUnavailable;

  return (
    <Container>
      <Volume />
      {bluetoothAvailable && (
        <StatefulPopover
          placement="bottom"
          content={({ close }) => <BluetoothList close={close} />}
          overrides={{
            Body: {
              style: {
                top: "10px",
                left: "-40px",
                zIndex: 100,
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
            <BluetoothIcon color={theme.colors.icon} />
          </button>
        </StatefulPopover>
      )}
      <StatefulPopover
        placement="bottom"
        content={({ close }) => <DeviceList close={close} />}
        overrides={{
          Body: {
            style: {
              top: "10px",
              left: "-70px",
              zIndex: 100,
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
              zIndex: 100,
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
          <List size={21} color={theme.colors.icon} />
        </Button>
      </StatefulPopover>
    </Container>
  );
};

export default RightMenu;
