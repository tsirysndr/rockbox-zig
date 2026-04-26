# rockbox-upnp

UPnP/DLNA support for Rockbox Zig. This crate provides three independent but
complementary features:

| Feature                              | What it does                                                                                        |
|--------------------------------------|-----------------------------------------------------------------------------------------------------|
| **Media Server** (ContentDirectory)  | Exposes the music library so UPnP control points (BubbleUPnP, Kodi, etc.) can browse and pull tracks |
| **MediaRenderer**                    | Lets control points push media to Rockbox (Rockbox becomes the speaker)                             |
| **PCM sink / WAV output**            | Streams live PCM audio as WAV-over-HTTP to an external UPnP renderer (Kodi, etc.)                  |

---

## How UPnP/DLNA works

### Protocol stack

```
Application      BubbleUPnP / Kodi / any DLNA app
                       │
     SOAP/XML over HTTP (AVTransport, ContentDirectory)
                       │
         UPnP Device Description  (XML, fetched from LOCATION URL)
                       │
       SSDP  (UDP multicast 239.255.255.250:1900)  ← device discovery
                       │
                     LAN
```

**SSDP** (Simple Service Discovery Protocol) is how devices announce themselves
and how control points find them. Each device sends periodic `NOTIFY` multicasts
and responds to `M-SEARCH` queries.

**Device Description** is an XML document (fetched via HTTP from the `LOCATION`
URL advertised in SSDP) that lists the device's friendly name, UDN (unique ID),
and the services it supports, with their control/event URLs.

**SOAP** is the RPC mechanism. Every action — browse library, set transport URI,
start playback — is a `POST` with an XML envelope to the service's `controlURL`.

### UPnP service roles

| UPnP term                       | Rockbox role   | Description                            |
|---------------------------------|----------------|----------------------------------------|
| MediaServer / ContentDirectory  | **server**     | Hosts the music library for browsing   |
| MediaRenderer / AVTransport     | **renderer**   | Receives push-play commands            |
| Control Point                   | *external app* | BubbleUPnP, Kodi, Foobar2000, …       |

---

## Feature 1 — Media Server (ContentDirectory)

When `upnp_server_enabled = true`, Rockbox starts a UPnP ContentDirectory
service that exposes the tag database (artists, albums, tracks).

### What a control point sees

```
Root
├── Artists
│   └── Daft Punk
│       └── Random Access Memories
│           └── Get Lucky          ← streamable via HTTP
├── Albums
│   └── ...
└── Tracks
    └── ...
```

Each track is served as a direct HTTP stream from the Rockbox GraphQL port
(`http://<ip>:<graphql_port>/tracks/<id>/stream`). Control points can play
individual tracks or enqueue them.

### Discovery flow

```
rockboxd starts
  └─ SSDP NOTIFY sent every 30 s to 239.255.255.250:1900
  └─ HTTP server on upnp_server_port (default 7878) answers:
       GET /device.xml          → UPnP device description
       POST /ContentDirectory   → Browse / Search actions (SOAP)
       GET /tracks/<id>/stream  → audio file bytes
```

A control point (e.g. BubbleUPnP) that receives the NOTIFY fetches
`/device.xml`, then issues `Browse` actions to walk the tree.

---

## Feature 2 — MediaRenderer (AVTransport)

When `upnp_renderer_enabled = true`, Rockbox advertises itself as a
`urn:schemas-upnp-org:device:MediaRenderer:1`. A control point can then push
any URI to Rockbox and tell it to play.

### Push-play flow (control point → Rockbox)

```
Control Point                     Rockbox (renderer)
     │                                   │
     │── SetAVTransportURI ─────────────>│  URL + DIDL-Lite metadata
     │                                   │  (title, artist, album, album art)
     │── Play ────────────────────────>  │  start playback
     │                                   │
     │── Pause / Stop / Seek ──────────> │
     │                                   │
     │── GetPositionInfo ─────────────>  │
     │<─ TrackDuration, RelTime ─────────│
```

**DIDL-Lite** (Digital Item Declaration Language Lite) is the XML format used
to carry track metadata in `SetAVTransportURI`. Rockbox parses the
`CurrentURIMetaData` field, XML-unescapes it, and extracts:

- `<dc:title>` — track title
- `<upnp:artist>` — artist name
- `<upnp:album>` — album name
- `<upnp:albumArtURI>` — album art URL
- `<res duration="H:MM:SS.mmm">` — track duration

Rockbox stores this and returns it in `GetPositionInfo` / `GetMediaInfo`
responses so control points can display a progress bar.

### Supported AVTransport actions

| Action                | Behaviour                                                                          |
|-----------------------|------------------------------------------------------------------------------------|
| `SetAVTransportURI`   | Store URI + parse DIDL-Lite metadata; open the stream in the Rockbox audio engine  |
| `Play`                | Start or resume playback                                                           |
| `Pause`               | Pause/resume toggle                                                                |
| `Stop`                | Stop playback; clear stored metadata                                               |
| `Seek`                | Seek to absolute time (REL_TIME target unit)                                       |
| `GetTransportInfo`    | Return current transport state (PLAYING / PAUSED_PLAYBACK / STOPPED)              |
| `GetPositionInfo`     | Return track URI, DIDL-Lite metadata, duration, elapsed time                      |
| `GetMediaInfo`        | Return current URI and DIDL-Lite metadata                                          |

---

## Feature 3 — PCM / WAV output sink

When `audio_output = "upnp"`, Rockbox encodes its live PCM output as a
continuous WAV stream and broadcasts it over HTTP, then commands an external
UPnP renderer to play that stream.

### Architecture

```
Rockbox audio engine (S16LE stereo PCM)
         │
  pcm_upnp_write()          ← called per DMA chunk
         │
  BroadcastBuffer (4 MB ring)
         │
  HTTP server (:upnp_http_port)
  GET /stream.wav  ──────────────────────────────────→  UPnP renderer
                                                        (Kodi, VLC, etc.)
         │
  AVTransport SOAP (SetAVTransportURI + Play)
  ┌───────────────────────────────────────────────────> upnp_renderer_url
  │  CurrentURI   = http://<local_ip>:<port>/stream.wav
  │  CurrentURIMetaData = DIDL-Lite with:
  │    title, artist, album, albumArtURI, duration
  │    (art URL: http://<local_ip>:<graphql_port>/covers/<filename>)
  │    (album art fetched from local SQLite library DB by track path)
  └
```

### Track-change metadata updates

A background task polls `current_track()` every 2 seconds. When the track path
changes it sends a new `SetAVTransportURI` with updated DIDL-Lite metadata
(`send_play = false` so the renderer does not restart the stream — the WAV
HTTP connection is continuous). This keeps the renderer's "Now Playing" display
accurate across track boundaries without interrupting audio.

### WAV stream format

```
Content-Type:  audio/wav
Transfer-Encoding: chunked (infinite stream)
Audio format:  PCM S16LE, 2 channels, sample rate = upnp_http_port
               standard 44-byte WAV header followed by raw PCM forever
```

The `BroadcastBuffer` is a 4 MB lock-free ring. Multiple HTTP clients can
connect simultaneously (each with an independent read cursor), lagging clients
skip forward rather than blocking the writer.

---

## Settings reference (`~/.config/rockbox.org/settings.toml`)

### Audio output — PCM sink to an external renderer

```toml
audio_output       = "upnp"

# URL of the renderer's AVTransport controlURL (required for metadata push)
upnp_renderer_url  = "http://192.168.1.x:7777/AVTransport/control"

# Port for the WAV HTTP broadcast server (default: 7879)
upnp_http_port     = 7879
```

To find `upnp_renderer_url`: on the renderer device look in its UPnP device
description XML (`LOCATION` from SSDP), find the `AVTransport` service block,
and read the `controlURL` element. Or enable the auto-scan (see below) and copy
the URL logged at startup.

### Media Server — expose library to control points

```toml
upnp_server_enabled  = true
upnp_server_port     = 7878   # HTTP port for ContentDirectory + device.xml
upnp_friendly_name   = "Rockbox"   # name shown in control points
```

### MediaRenderer — let control points push media to Rockbox

```toml
upnp_renderer_enabled = true
upnp_renderer_port    = 7880   # HTTP port for AVTransport + device.xml
upnp_friendly_name    = "Rockbox"
```

### All UPnP settings at a glance

| Key | Type | Default | Description |
|---|---|---|---|
| `audio_output` | string | `"builtin"` | Set to `"upnp"` to use the PCM sink |
| `upnp_renderer_url` | string | — | AVTransport controlURL of the target renderer |
| `upnp_http_port` | integer | `7879` | Port for the WAV broadcast HTTP server |
| `upnp_server_enabled` | bool | `false` | Start the ContentDirectory media server |
| `upnp_server_port` | integer | `7878` | HTTP port for the media server |
| `upnp_renderer_enabled` | bool | `false` | Start the MediaRenderer |
| `upnp_renderer_port` | integer | `7880` | HTTP port for the renderer |
| `upnp_friendly_name` | string | `"Rockbox"` | Display name shown to control points |

---

## Using Kodi as a UPnP renderer

1. In Kodi: **Settings → Services → UPnP/DLNA** → enable "Allow remote control via UPnP".

2. Find Kodi's AVTransport URL. Either:
   - Run `rockboxd` with `RUST_LOG=info` and look for
     `upnp scan: found renderer "Kodi" av=http://...`  
     (Rockbox scans the LAN for renderers at startup).
   - Or browse Kodi's device description at
     `http://<kodi-ip>:1234/upnpms/device.xml` and find the
     `AVTransport` `controlURL`.

3. Set `settings.toml`:
   ```toml
   audio_output      = "upnp"
   upnp_renderer_url = "http://192.168.1.x:1234/upnpms/event"
   ```

4. Start `rockboxd` and play a track. Kodi will show the track title, artist,
   album, and album art.

---

## LAN renderer auto-discovery

At startup, Rockbox sends an SSDP `M-SEARCH` for
`urn:schemas-upnp-org:device:MediaRenderer:1` and logs all discovered
renderers. The scan result is also available through the HTTP/gRPC API (device
list endpoint), so the UI can offer a renderer picker.

Discovery sequence:

```
rockboxd                            LAN
   │── M-SEARCH (UDP multicast) ───>│
   │<─ 200 OK  LOCATION: http://... │  (one reply per renderer)
   │── GET <LOCATION>  ─────────────>│  fetch device description XML
   │<─ device.xml ──────────────────│
   │   parse: friendlyName, UDN, AVTransport controlURL
   │   → Device { app: UPNP_DLNA, base_url: controlURL }
```

---

## Crate layout

```
src/
  lib.rs         — FFI exports (pcm_upnp_*), BroadcastBuffer, global state,
                   AVTransport SOAP client, track-change monitor
  db.rs          — SQLite helpers (open_pool, track_by_path, all_tracks, …)
  didl.rs        — DIDL-Lite XML parser (for incoming SetAVTransportURI)
  format.rs      — WAV header builder
  pcm_server.rs  — HTTP broadcast server (one goroutine per client)
  renderer.rs    — MediaRenderer HTTP handler (AVTransport SOAP endpoint)
  scan.rs        — SSDP M-SEARCH + device description probe
  server.rs      — ContentDirectory HTTP handler (Browse/Search SOAP)
  ssdp.rs        — SSDP NOTIFY announcer
```
