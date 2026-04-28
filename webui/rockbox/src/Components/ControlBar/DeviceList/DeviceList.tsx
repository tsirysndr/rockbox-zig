import { useTheme } from "@emotion/react";
import styled from "@emotion/styled";
import { ListItem, ListItemLabel } from "baseui/list";
import { FC } from "react";
import { Laptop } from "@styled-icons/ionicons-outline";
import { Kodi, Airplayaudio, Chromecast } from "@styled-icons/simple-icons";
import { Speaker } from "@styled-icons/remix-fill";
import { Radio, HardDrive, Cast } from "@styled-icons/feather";
import {
  Container,
  CurrentDevice,
  CurrentDeviceName,
  CurrentDeviceWrapper,
  Disconnect,
  Icon,
  IconWrapper,
  List,
  Placeholder,
  Title,
} from "./styles";

export type Device = {
  id: string;
  name: string;
  /** Maps to the `app` field from the server (e.g. "builtin", "fifo", "squeezelite",
   *  "AirPlay", "Chromecast", "UPnP/DLNA", "xbmc") */
  type: string;
  isConnected: boolean;
  isCurrentDevice: boolean;
};

export type ArtworkProps = {
  icon?: string;
  color?: string;
};

const iconColors: Record<string, string> = {
  builtin:      "#28fce3",
  fifo:         "#9090ff",
  squeezelite:  "#ffa028",
  snapcast:     "#a0e040",
  Snapcast:     "#a0e040",
  xbmc:         "#28cbfc",
  AirPlay:      "#ff00c3",
  airplay:      "#ff00c3",
  Chromecast:   "#28cbfc",
  chromecast:   "#28cbfc",
  "UPnP/DLNA":  "#ff00c3",
  dlna:         "#ff00c3",
};

const bgColors: Record<string, string> = {
  builtin:      "rgba(40, 252, 227, 0.09)",
  fifo:         "rgba(144, 144, 255, 0.10)",
  squeezelite:  "rgba(255, 160, 40, 0.10)",
  snapcast:     "rgba(160, 224, 64, 0.10)",
  Snapcast:     "rgba(160, 224, 64, 0.10)",
  xbmc:         "rgba(40, 203, 252, 0.08)",
  AirPlay:      "rgba(255, 0, 195, 0.06)",
  airplay:      "rgba(255, 0, 195, 0.06)",
  Chromecast:   "rgba(40, 203, 252, 0.08)",
  chromecast:   "rgba(40, 203, 252, 0.08)",
  "UPnP/DLNA":  "rgba(255, 0, 195, 0.06)",
  dlna:         "rgba(255, 0, 195, 0.06)",
};

const Artwork: FC<ArtworkProps> = ({ icon, color }) => {
  const theme = useTheme();
  const c = iconColors[icon ?? ""] ?? theme.colors.text;
  return (
    <Icon color={color}>
      {icon === "builtin"     && <HardDrive size={18} color={c} />}
      {icon === "fifo"        && <Radio     size={18} color={c} />}
      {icon === "squeezelite" && <Cast      size={18} color={c} />}
      {(icon === "snapcast" || icon === "Snapcast") && (
        <Radio size={18} color={c} />
      )}
      {icon === "xbmc"        && <Kodi      size={18} color={c} />}
      {(icon === "AirPlay" || icon === "airplay") && (
        <Airplayaudio size={18} color={c} />
      )}
      {(icon === "Chromecast" || icon === "chromecast") && (
        <Chromecast size={18} color={c} />
      )}
      {(icon === "UPnP/DLNA" || icon === "dlna") && (
        <Speaker size={18} color={c} />
      )}
    </Icon>
  );
};

const DeviceName = styled.div`
  font-size: 14px;
`;

const ActiveDot = styled.div`
  width: 7px;
  height: 7px;
  border-radius: 50%;
  background-color: #28fce3;
  flex-shrink: 0;
  margin-right: 8px;
`;

export type DeviceListProps = {
  currentCastDevice?: Device | null;
  castDevices: Device[];
  connectToCastDevice: (deviceId: string) => void;
  disconnectFromCastDevice: () => void;
  close: () => void;
  loading: boolean;
};

const DeviceList: FC<DeviceListProps> = ({
  castDevices,
  close,
  connectToCastDevice,
  disconnectFromCastDevice,
  currentCastDevice,
  loading,
}) => {
  const theme = useTheme();

  const _onConnectToCastDevice = (deviceId: string) => {
    connectToCastDevice(deviceId);
    close();
  };

  const _onDisconnectFromCastDevice = () => {
    disconnectFromCastDevice();
    close();
  };

  // The active device is either what DeviceState tracks or derived from the list.
  const activeDevice =
    currentCastDevice ??
    castDevices.find((d) => d.isCurrentDevice) ??
    null;

  return (
    <Container>
      {/* Current device header */}
      <CurrentDeviceWrapper>
        <IconWrapper>
          {activeDevice ? (
            <Artwork
              icon={activeDevice.type}
              color={bgColors[activeDevice.type]}
            />
          ) : (
            <Laptop size={30} color="#6F00FF" />
          )}
        </IconWrapper>
        <div style={{ flex: 1 }}>
          <CurrentDevice>Current device</CurrentDevice>
          <CurrentDeviceName>
            {activeDevice ? activeDevice.name : "Rockbox (Built-in)"}
          </CurrentDeviceName>
        </div>
        {currentCastDevice && (
          <Disconnect onClick={_onDisconnectFromCastDevice}>
            disconnect
          </Disconnect>
        )}
      </CurrentDeviceWrapper>

      {!loading && castDevices.length > 0 && (
        <Title>Output device</Title>
      )}

      <List>
        {castDevices.length === 0 && !loading && (
          <Placeholder>
            No devices found. Make sure your devices are on the same network.
          </Placeholder>
        )}
        {castDevices.map((device) => (
          <div
            key={device.id}
            onClick={() => {
              if (!device.isCurrentDevice) {
                _onConnectToCastDevice(device.id);
              }
            }}
          >
            <ListItem
              artwork={() => (
                <Artwork icon={device.type} color={bgColors[device.type]} />
              )}
              endEnhancer={() =>
                device.isCurrentDevice ? <ActiveDot /> : null
              }
              overrides={{
                Root: {
                  style: {
                    cursor: device.isCurrentDevice ? "default" : "pointer",
                    backgroundColor: "transparent",
                    ":hover": {
                      backgroundColor: device.isCurrentDevice
                        ? "transparent"
                        : theme.colors.hover,
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
                <DeviceName>{device.name}</DeviceName>
              </ListItemLabel>
            </ListItem>
          </div>
        ))}
      </List>
    </Container>
  );
};

export default DeviceList;
