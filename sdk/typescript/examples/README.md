# Examples

Runnable sample programs for `@rockbox-zig/sdk`.

## Prerequisites

- `bun` installed (https://bun.sh)
- `rockboxd` running and reachable at `http://localhost:6062/graphql`
  ```sh
  ./zig/zig-out/bin/rockboxd
  ```
- SDK dependencies installed (run once from `sdk/typescript/`):
  ```sh
  bun install
  ```

## Running an example

From the `sdk/typescript/` directory:

```sh
bun run examples/01-basic-playback.ts
```

Override the host/port with environment variables:

```sh
ROCKBOX_HOST=192.168.1.42 ROCKBOX_PORT=6062 bun run examples/01-basic-playback.ts
```

## Index

| File                                  | Demonstrates                                            |
|---------------------------------------|---------------------------------------------------------|
| `01-basic-playback.ts`                | Status, transport controls, current track               |
| `02-now-playing.ts`                   | Real-time WebSocket subscriptions                       |
| `03-library-search.ts`                | Search the library and play results                     |
| `04-queue-management.ts`              | Inspect and manipulate the playback queue               |
| `05-saved-playlists.ts`               | Create, edit, and play saved playlists                  |
| `06-smart-playlist.ts`                | Build smart playlists from rule sets                    |
| `07-volume-control.ts`                | Read `VolumeInfo` and adjust relative volume            |
| `08-eq-config.ts`                     | Configure the equalizer and replaygain                  |
| `09-browse-filesystem.ts`             | Walk `music_dir` like a tree                            |
| `10-browse-upnp.ts`                   | Discover and browse UPnP media servers                  |
| `11-bluetooth.ts`                     | Scan, connect, and disconnect Bluetooth devices (Linux) |
| `12-devices.ts`                       | List and switch Chromecast / AirPlay output sinks       |
| `13-plugin-sleep-timer.ts`            | Plugin: stop playback after N minutes                   |
| `14-plugin-scrobbler.ts`              | Plugin: log every fully-played track                    |
| `15-cli-remote.ts`                    | Tiny interactive remote control in the terminal         |

Each example is self-contained — pick the one closest to what you need, copy
it into your project, and adapt.
