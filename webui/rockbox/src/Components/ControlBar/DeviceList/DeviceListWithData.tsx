import { FC, useEffect, useMemo } from "react";
import DeviceList from "./DeviceList";
import {
  useConnectToDeviceMutation,
  useDisconnectFromDeviceMutation,
  useGetDeviceQuery,
  useGetDevicesQuery,
} from "../../../Hooks/GraphQL";
import { useRecoilState } from "recoil";
import { deviceState } from "./DeviceState";
import { controlBarState } from "../ControlBarState";

export type DeviceListWithDataProps = {
  close: () => void;
};

const DeviceListWithData: FC<DeviceListWithDataProps> = ({ close }) => {
  const [, setControlBarState] = useRecoilState(controlBarState);
  const [device, setDeviceState] = useRecoilState(deviceState);
  const { data: currentDevice } = useGetDeviceQuery({
    variables: { id: "current" },
    fetchPolicy: "network-only",
  });
  const { data, loading } = useGetDevicesQuery();
  const [connect] = useConnectToDeviceMutation();
  const [disconnect] = useDisconnectFromDeviceMutation();
  const devices = useMemo(() => {
    if (loading || !data) {
      return [];
    }
    return (data.devices || []).map((x) => ({
      id: x.id,
      name: x.name,
      type: x.app,
      isConnected: x.isConnected,
    }));
  }, [data, loading]);

  useEffect(() => {
    if (currentDevice) {
      if (!currentDevice.device) {
        setDeviceState({
          currentDevice: null,
        });
        return;
      }
      setDeviceState({
        currentDevice: {
          id: currentDevice.device.id || "",
          name: currentDevice.device.name || "",
          type: currentDevice.device.app || "",
          isConnected: currentDevice.device.isConnected || false,
        },
      });
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [currentDevice]);

  const connectToCastDevice = async (id: string) => {
    await connect({ variables: { id } });
    setControlBarState((state) => ({
      ...state,
      nowPlaying: undefined,
    }));
  };

  const disconnectDevice = async () => {
    await disconnect({ variables: { id: currentDevice?.device?.id || "" } });
    // reload page to reset the state
    window.location.reload();
  };

  return (
    <>
      {
        <DeviceList
          loading={loading}
          castDevices={devices}
          connectToCastDevice={connectToCastDevice}
          disconnectFromCastDevice={disconnectDevice}
          close={close}
          currentCastDevice={device.currentDevice}
        />
      }
    </>
  );
};

export default DeviceListWithData;
