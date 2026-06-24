defmodule Rockbox.MixProject do
  use Mix.Project

  @version "0.1.0"
  @source_url "https://github.com/tsirysndr/rockboxd"

  def project do
    [
      app: :rockbox_ex,
      version: @version,
      elixir: "~> 1.15",
      start_permanent: Mix.env() == :prod,
      deps: deps(),
      description: description(),
      package: package(),
      docs: docs(),
      name: "Rockbox",
      source_url: @source_url
    ]
  end

  def application do
    [
      extra_applications: [:logger],
      mod: {Rockbox.Application, []}
    ]
  end

  defp deps do
    [
      {:req, "~> 0.5"},
      {:jason, "~> 1.4"},
      {:mint_web_socket, "~> 1.0"},
      {:ex_doc, "~> 0.34", only: :dev, runtime: false}
    ]
  end

  defp description do
    "Idiomatic Elixir SDK for Rockbox — pipe-friendly, builder-friendly, with real-time event subscriptions and a plugin system."
  end

  defp package do
    [
      maintainers: ["Tsiry Sandratraina"],
      licenses: ["MIT"],
      links: %{"GitHub" => @source_url, "Rockbox" => "https://www.rockbox.org"},
      files: ~w(lib mix.exs README.md .formatter.exs)
    ]
  end

  defp docs do
    [
      main: "Rockbox",
      extras: ["README.md"],
      groups_for_modules: [
        Core: [Rockbox, Rockbox.Client, Rockbox.Error],
        APIs: [
          Rockbox.Playback,
          Rockbox.Library,
          Rockbox.Queue,
          Rockbox.SavedPlaylists,
          Rockbox.SmartPlaylists,
          Rockbox.Sound,
          Rockbox.Settings,
          Rockbox.System,
          Rockbox.Browse,
          Rockbox.Devices,
          Rockbox.Bluetooth
        ],
        Events: [Rockbox.Events, Rockbox.Socket],
        Plugins: [Rockbox.Plugin, Rockbox.Plugins],
        Builders: [Rockbox.SmartPlaylist.Rules]
      ]
    ]
  end
end
