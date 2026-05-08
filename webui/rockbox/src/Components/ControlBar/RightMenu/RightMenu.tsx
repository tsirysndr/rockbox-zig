import { FC } from "react";
import { List } from "@styled-icons/entypo";
import { StatefulPopover } from "baseui/popover";
import PlayQueue from "../PlayQueue";
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
  const { isError: bluetoothUnavailable } = useGetBluetoothDevicesQuery({
    retry: false,
  });
  const bluetoothAvailable = !bluetoothUnavailable;

  return (
    <div className="flex flex-row flex-[0.3] h-[60px] min-w-[160px] items-center justify-end gap-3">
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
                backgroundColor: "var(--theme-popover-background)",
              },
            },
          }}
        >
          <button className="bg-transparent cursor-pointer border-0 flex items-center justify-center hover:opacity-60">
            <BluetoothIcon color="var(--theme-icon)" />
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
              backgroundColor: "var(--theme-popover-background)",
            },
          },
        }}
      >
        <button className="bg-transparent cursor-pointer border-0 flex items-center justify-center hover:opacity-60">
          <Speaker size={18} color="var(--theme-icon)" />
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
              backgroundColor: "var(--theme-popover-background)",
            },
          },
        }}
      >
        <button className="bg-transparent cursor-pointer border-0 flex items-center justify-center hover:opacity-60">
          <List size={21} color="var(--theme-icon)" />
        </button>
      </StatefulPopover>
    </div>
  );
};

export default RightMenu;
