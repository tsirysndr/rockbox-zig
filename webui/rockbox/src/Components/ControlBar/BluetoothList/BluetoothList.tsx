import { ListItem, ListItemLabel } from "baseui/list";
import { FC } from "react";
import { BluetoothDeviceGql } from "../../../Hooks/GraphQL";

const ACCENT = "#1a91ff";

const DeviceNameText: FC<{ children: React.ReactNode }> = ({ children }) => (
  <div className="text-[13px]">{children}</div>
);

const CheckMark: FC<{ children: React.ReactNode }> = ({ children }) => (
  <span className="text-[#28fce3] text-xs ml-[6px]">{children}</span>
);

const BluetoothIcon: FC = () => (
  <div
    style={{
      width: 30,
      height: 30,
      borderRadius: 6,
      background: "rgba(26,145,255,0.12)",
      display: "flex",
      alignItems: "center",
      justifyContent: "center",
    }}
  >
    <svg viewBox="0 0 24 24" width={14} height={14} fill={ACCENT}>
      <path d="M17.71 7.71L12 2h-1v7.59L6.41 5 5 6.41 10.59 12 5 17.59 6.41 19 11 14.41V22h1l5.71-5.71-4.3-4.29 4.3-4.29zM13 5.83l1.88 1.88L13 9.59V5.83zm1.88 10.46L13 18.17v-3.76l1.88 1.88z" />
    </svg>
  </div>
);

export type BluetoothListProps = {
  devices: BluetoothDeviceGql[];
  loading: boolean;
  connect: (address: string) => void;
  disconnect: (address: string) => void;
  close: () => void;
};

const BluetoothList: FC<BluetoothListProps> = ({
  devices,
  loading,
  connect,
  disconnect,
  close,
}) => {
  const handleTap = (device: BluetoothDeviceGql) => {
    if (device.connected) {
      disconnect(device.address);
    } else {
      connect(device.address);
    }
    close();
  };

  return (
    <div className="max-h-[calc(100vh-153px)] pt-[15px] pb-[15px] overflow-y-auto w-[280px] min-h-[120px]">
      {devices.length > 0 && !loading && (
        <div className="m-[10px] mx-[25px] font-[RockfordSansBold]">Bluetooth speakers</div>
      )}
      <div className="max-h-[calc(100vh-273px)] px-[15px] overflow-y-auto min-h-[80px]">
        {!loading && devices.length === 0 && (
          <div className="flex items-center justify-center h-[120px] text-center px-5 text-sm">
            No bluetooth devices found.
          </div>
        )}
        {devices.map((device) => (
          <div key={device.address} onClick={() => handleTap(device)}>
            <ListItem
              artwork={() => <BluetoothIcon />}
              endEnhancer={() =>
                device.connected ? <CheckMark>✓</CheckMark> : null
              }
              overrides={{
                Root: {
                  style: {
                    cursor: "pointer",
                    backgroundColor: "transparent",
                    ":hover": {
                      backgroundColor: "var(--theme-hover)",
                    },
                    borderRadius: "5px",
                  },
                },
                Content: {
                  style: { borderBottom: "none" },
                },
              }}
            >
              <ListItemLabel>
                <DeviceNameText>{device.name || device.address}</DeviceNameText>
              </ListItemLabel>
            </ListItem>
          </div>
        ))}
      </div>
    </div>
  );
};

export default BluetoothList;
