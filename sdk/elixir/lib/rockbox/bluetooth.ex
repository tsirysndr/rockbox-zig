defmodule Rockbox.Bluetooth do
  @moduledoc """
  Bluetooth device management (Linux only — backed by BlueZ).

  Calls return `{:error, %Rockbox.GraphQLError{}}` on non-Linux hosts with the
  message `"Bluetooth is only supported on Linux"`.
  """

  alias Rockbox.{BluetoothDevice, Client, Transport, Util}

  @fragment ~S"""
    fragment BluetoothDeviceFields on BluetoothDevice {
      address name paired trusted connected rssi
    }
  """

  @spec devices(Client.t()) :: {:ok, [BluetoothDevice.t()]} | {:error, Exception.t()}
  def devices(client) do
    query =
      @fragment <> "query BluetoothDevices { bluetoothDevices { ...BluetoothDeviceFields } }"

    with {:ok, %{"bluetoothDevices" => list}} <- Transport.execute(client, query) do
      {:ok, Util.to_struct_list(BluetoothDevice, list)}
    end
  end

  @spec devices!(Client.t()) :: [BluetoothDevice.t()]
  def devices!(client), do: bang(devices(client))

  @doc "Scan for nearby devices. `timeout_secs` defaults to the firmware default."
  @spec scan(Client.t(), pos_integer() | nil) ::
          {:ok, [BluetoothDevice.t()]} | {:error, Exception.t()}
  def scan(client, timeout_secs \\ nil) do
    query =
      @fragment <>
        "mutation BluetoothScan($timeoutSecs: Int) { bluetoothScan(timeoutSecs: $timeoutSecs) { ...BluetoothDeviceFields } }"

    with {:ok, %{"bluetoothScan" => list}} <-
           Transport.execute(client, query, %{timeout_secs: timeout_secs}) do
      {:ok, Util.to_struct_list(BluetoothDevice, list)}
    end
  end

  @spec connect(Client.t(), String.t()) :: :ok | {:error, Exception.t()}
  def connect(client, address),
    do:
      void(
        Transport.execute(
          client,
          "mutation BluetoothConnect($address: String!) { bluetoothConnect(address: $address) }",
          %{address: address}
        )
      )

  @spec disconnect(Client.t(), String.t()) :: :ok | {:error, Exception.t()}
  def disconnect(client, address),
    do:
      void(
        Transport.execute(
          client,
          "mutation BluetoothDisconnect($address: String!) { bluetoothDisconnect(address: $address) }",
          %{address: address}
        )
      )

  defp void({:ok, _}), do: :ok
  defp void(err), do: err

  defp bang({:ok, value}), do: value
  defp bang({:error, exception}), do: raise(exception)
end
