# Rockbox Daemon — Documentation

The Mintlify source for [https://rockboxzig.mintlify.app](https://rockboxzig.mintlify.app).

## Layout

```
mintlify/
├─ docs.json              # navigation, theme, anchors
├─ index.mdx              # landing page
├─ quickstart.mdx
├─ installation.mdx
├─ configuration.mdx
├─ audio-output/          # builtin / snapcast / airplay / squeezelite / chromecast / upnp
├─ audio-settings/        # EQ, DSP, ReplayGain, crossfade
├─ clients/               # web, desktop, MPD, MPRIS
├─ architecture/          # build system, PCM sinks
├─ reference/             # CLI, ports, settings.toml, troubleshooting, FAQ
├─ api-reference/
│   ├─ introduction.mdx
│   ├─ openapi.json       # synced from crates/server/openapi.json
│   ├─ rest/overview.mdx
│   ├─ graphql/overview.mdx
│   ├─ grpc/overview.mdx
│   └─ mpd/overview.mdx
└─ sdks/                  # TypeScript / Python / Ruby / Elixir / Clojure / Gleam
```

## Local preview

```sh
npm i -g mint
mint dev
```

The preview lives at [http://localhost:3000](http://localhost:3000) and
reloads on save.

## Keeping the OpenAPI spec in sync

`api-reference/openapi.json` mirrors `crates/server/openapi.json`. After
adding or changing an HTTP route, edit
`crates/server/openapi.json` and copy it over:

```sh
cp crates/server/openapi.json mintlify/api-reference/openapi.json
```

Mintlify auto-generates one page per operation under the **HTTP REST**
group on every build.

## Validating before pushing

```sh
mint broken-links
```

## Deployment

The Mintlify GitHub app deploys this directory automatically on push to
the default branch. No manual step needed.
