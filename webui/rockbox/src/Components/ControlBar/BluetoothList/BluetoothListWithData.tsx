import { FC } from "react";
import BluetoothList from "./BluetoothList";
import {
  useGetBluetoothDevicesQuery,
  useBluetoothConnectMutation,
  useBluetoothDisconnectMutation,
} from "../../../Hooks/GraphQL";

export type BluetoothListWithDataProps = {
  close: () => void;
};

const BluetoothListWithData: FC<BluetoothListWithDataProps> = ({ close }) => {
  const { data, isLoading, refetch } = useGetBluetoothDevicesQuery({
    retry: false,
  });
  const { mutateAsync: connectAsync } = useBluetoothConnectMutation();
  const { mutateAsync: disconnectAsync } = useBluetoothDisconnectMutation();

  const devices = data?.bluetoothDevices ?? [];

  const connect = async (address: string) => {
    await connectAsync({ address });
    await refetch();
  };

  const disconnect = async (address: string) => {
    await disconnectAsync({ address });
    await refetch();
  };

  return (
    <BluetoothList
      devices={devices}
      loading={isLoading}
      connect={connect}
      disconnect={disconnect}
      close={close}
    />
  );
};

export default BluetoothListWithData;
