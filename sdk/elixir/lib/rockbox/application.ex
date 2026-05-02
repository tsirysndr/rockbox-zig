defmodule Rockbox.Application do
  @moduledoc false

  use Application

  @impl true
  def start(_type, _args) do
    children = [
      {Registry, keys: :duplicate, name: Rockbox.Subscribers},
      {Registry, keys: :unique, name: Rockbox.Sockets},
      {DynamicSupervisor, strategy: :one_for_one, name: Rockbox.SocketSupervisor},
      Rockbox.Plugins
    ]

    Supervisor.start_link(children, strategy: :one_for_one, name: Rockbox.Supervisor)
  end
end
