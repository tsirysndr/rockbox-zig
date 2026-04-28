import { FC, useMemo } from "react";
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
  const [device] = useRecoilState(deviceState);
  const { data: currentDevice, refetch: refetchCurrentDevice } = useGetDeviceQuery({ id: "current" });
  const { data, isLoading, refetch: refetchDevices } = useGetDevicesQuery();
  const { mutateAsync: connectAsync } = useConnectToDeviceMutation();
  const { mutateAsync: disconnectAsync } = useDisconnectFromDeviceMutation();
  const devices = useMemo(() => {
    if (isLoading || !data) {
      return [];
    }
    return (data.devices || []).map((x) => ({
      id: x.id,
      name: x.name,
      type: x.app,
      isConnected: x.isConnected,
      isCurrentDevice: x.isCurrentDevice ?? false,
    }));
  }, [data, isLoading]);

  const connectToCastDevice = async (id: string) => {
    await connectAsync({ id });
    await Promise.all([refetchCurrentDevice(), refetchDevices()]);
    setControlBarState((state) => ({
      ...state,
      nowPlaying: undefined,
    }));
  };

  const disconnectDevice = async () => {
    await disconnectAsync({ id: currentDevice?.device?.id || "" });
    // reload page to reset the state
    window.location.reload();
  };

  return (
    <>
      {
        <DeviceList
          loading={isLoading}
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
