import { useTheme } from "@emotion/react";
import styled from "@emotion/styled";
import { ListItem, ListItemLabel } from "baseui/list";
import { FC } from "react";
import { MusicPlayer } from "@styled-icons/bootstrap";
import { Laptop } from "@styled-icons/ionicons-outline";
import { Kodi, Airplayaudio, Chromecast } from "@styled-icons/simple-icons";
import { Speaker } from "@styled-icons/remix-fill";
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
  type: string;
  isConnected: boolean;
};

export type ArtworkProps = {
  icon?: string;
  color?: string;
};

const Artwork: FC<ArtworkProps> = (
  { icon, color } = {
    icon: "music-player",
  }
) => {
  const theme = useTheme();
  return (
    <Icon color={color}>
      {icon === "music-player" && <MusicPlayer size={18} color="#28fce3" />}
      {icon === "xbmc" && <Kodi size={18} color="#28cbfc" />}
      {icon === "airplay" && <Airplayaudio size={18} color={"#ff00c3"} />}
      {icon === "chromecast" && (
        <Chromecast size={18} color={theme.colors.text} />
      )}
      {icon === "dlna" && <Speaker size={18} color={"#ff00c3"} />}
    </Icon>
  );
};

const DeviceName = styled.div`
  font-size: 14px;
  color: "#fe099c";
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
  const colors: {
    [key: string]: string;
  } = {
    "music-player": "rgba(40, 252, 227, 0.088)",
    xbmc: "rgba(40, 203, 252, 0.082)",
    airplay: "rgba(255, 0, 195, 0.063)",
    dlna: "rgba(255, 0, 195, 0.063)",
  };

  const _onConnectToCastDevice = (deviceId: string) => {
    connectToCastDevice(deviceId);
    close();
  };

  const _onDisconnectFromCastDevice = () => {
    disconnectFromCastDevice();
    close();
  };

  return (
    <Container>
      <CurrentDeviceWrapper>
        <IconWrapper>
          <Laptop size={30} color={"#fe099c"} />
        </IconWrapper>
        <div style={{ flex: 1 }}>
          <CurrentDevice>Current device</CurrentDevice>
          <CurrentDeviceName>
            {currentCastDevice ? currentCastDevice.name : "Rockbox"}
          </CurrentDeviceName>
        </div>
        {currentCastDevice && (
          <Disconnect onClick={_onDisconnectFromCastDevice}>
            disconnect
          </Disconnect>
        )}
      </CurrentDeviceWrapper>
      {!loading && <Title>Select another output device</Title>}
      <List>
        {castDevices.length === 0 && !loading && (
          <Placeholder>
            No devices found. Please make sure your device is connected to the
            same network as this device.
          </Placeholder>
        )}
        {castDevices.map((device) => (
          <div
            key={device.id}
            onClick={() => _onConnectToCastDevice(device.id)}
          >
            <ListItem
              key={device.id}
              artwork={() => (
                <Artwork icon={device.type} color={colors[device.type]} />
              )}
              overrides={{
                Root: {
                  style: {
                    cursor: "pointer",
                    ":hover": {
                      backgroundColor: theme.colors.hover,
                    },
                    borderRadius: "5px",
                  },
                },
                Content: {
                  style: {
                    borderBottom: "none",
                  },
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
