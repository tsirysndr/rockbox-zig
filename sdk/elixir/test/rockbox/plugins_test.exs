defmodule Rockbox.PluginsTest do
  use ExUnit.Case, async: false

  defmodule HelloPlugin do
    @behaviour Rockbox.Plugin

    @impl true
    def name, do: "hello"
    @impl true
    def version, do: "0.1.0"
    @impl true
    def description, do: "Test plugin"

    @impl true
    def install(ctx) do
      {pid, _} = :persistent_term.get({__MODULE__, :test_pid}, {nil, nil})
      if pid, do: send(pid, {:installed, ctx.client})
      {:ok, %{installed_at: System.system_time()}}
    end

    @impl true
    def uninstall(_state) do
      {pid, _} = :persistent_term.get({__MODULE__, :test_pid}, {nil, nil})
      if pid, do: send(pid, :uninstalled)
      :ok
    end
  end

  setup do
    :persistent_term.put({HelloPlugin, :test_pid}, {self(), make_ref()})
    Rockbox.Plugins.uninstall("hello")

    on_exit(fn ->
      Rockbox.Plugins.uninstall("hello")
    end)

    :ok
  end

  test "install / list / uninstall" do
    client = Rockbox.new()

    assert :ok = Rockbox.use_plugin(client, HelloPlugin)
    assert_receive {:installed, %Rockbox.Client{}}

    names = Enum.map(Rockbox.installed_plugins(), & &1.module.name())
    assert "hello" in names

    assert :ok = Rockbox.unuse_plugin("hello")
    assert_receive :uninstalled
  end

  test "double-install is rejected" do
    client = Rockbox.new()
    assert :ok = Rockbox.use_plugin(client, HelloPlugin)
    assert_receive {:installed, _}
    assert {:error, :already_installed} = Rockbox.use_plugin(client, HelloPlugin)
  end
end
