defmodule Rockbox.Devices do
  @moduledoc """
  Discovered remote output sinks (Chromecast, AirPlay receivers, …) advertised
  over mDNS. Connecting switches the active PCM output to that device.
  """

  alias Rockbox.{Client, Device, Transport, Util}

  @fields "id name host ip port service app isConnected baseUrl isCastDevice isSourceDevice isCurrentDevice"

  @spec list(Client.t()) :: {:ok, [Device.t()]} | {:error, Exception.t()}
  def list(client) do
    with {:ok, %{"devices" => list}} <-
           Transport.execute(client, "query Devices { devices { #{@fields} } }") do
      {:ok, Util.to_struct_list(Device, list)}
    end
  end

  @spec list!(Client.t()) :: [Device.t()]
  def list!(client), do: bang(list(client))

  @spec get(Client.t(), String.t()) :: {:ok, Device.t() | nil} | {:error, Exception.t()}
  def get(client, id) do
    with {:ok, %{"device" => raw}} <-
           Transport.execute(
             client,
             "query Device($id: String!) { device(id: $id) { #{@fields} } }",
             %{id: id}
           ) do
      {:ok, Util.to_struct(Device, raw)}
    end
  end

  @spec connect(Client.t(), String.t()) :: :ok | {:error, Exception.t()}
  def connect(client, id),
    do:
      void(
        Transport.execute(
          client,
          "mutation ConnectDevice($id: String!) { connect(id: $id) }",
          %{id: id}
        )
      )

  @spec disconnect(Client.t(), String.t()) :: :ok | {:error, Exception.t()}
  def disconnect(client, id),
    do:
      void(
        Transport.execute(
          client,
          "mutation DisconnectDevice($id: String!) { disconnect(id: $id) }",
          %{id: id}
        )
      )

  defp void({:ok, _}), do: :ok
  defp void(err), do: err

  defp bang({:ok, value}), do: value
  defp bang({:error, exception}), do: raise(exception)
end
