# rockbox-s3-admin

Web admin UI for the rockboxd S3-compatible API. Single-page React app that
talks straight to the S3 endpoint on the same origin (no separate JSON API on
the Rust side) and is embedded into the daemon at build time via
`rust-embed`, then served at `/admin/`.

## What it does

- Logs in with an S3 access key / secret key pair (verified against the
  daemon by issuing a SigV4-signed `HeadBucket music`).
- Browses the `music` bucket as a filesystem (prefix + delimiter listing,
  prefix navigation, file metadata).
- Uploads new audio files via presigned PUT with progress events.
- Downloads files via short-lived presigned GET URLs.
- Deletes objects; the daemon's library watcher picks the change up and
  reconciles the SQLite music database.
- Settings page surfaces the active endpoint and signed-in credentials.

All requests go to `window.location.origin`, region `us-east-1`, bucket
`music`, `forcePathStyle: true` — matching the constants enforced by
`crates/s3/`.

## Stack

- **Vite 8** + **React 19** + **TypeScript**
- **TanStack Router** (file-based routes under `src/routes/`, generated
  `routeTree.gen.ts`)
- **TanStack Query** for data fetching
- **Jotai** for auth/session state
- **Tailwind CSS 4** + **FlyonUI** components
- **AWS SDK v3** (`@aws-sdk/client-s3`, `@aws-sdk/s3-request-presigner`)
- **Oxlint** for linting

## Layout

```
src/
├── main.tsx                main entrypoint
├── routes/
│   ├── __root.tsx          root layout
│   ├── login.tsx           credential entry → HeadBucket verify
│   ├── _app.tsx            authenticated shell (sidebar + topbar)
│   ├── _app.index.tsx      dashboard
│   ├── _app.browser.tsx    object browser
│   ├── _app.upload.tsx     upload page (presigned PUT + XHR progress)
│   └── _app.settings.tsx   session / endpoint info
├── components/             Sidebar, Topbar
├── atoms/                  auth, ui (Jotai)
└── lib/
    ├── s3.ts               S3Client wiring + list/get/put/delete helpers
    └── format.ts           byte/date formatting
```

## Develop

```sh
bun install
bun run dev        # vite dev server with HMR
bun run lint       # oxlint
bun run build      # tsr generate && tsc -b && vite build → dist/
bun run preview    # serve dist/ locally
```

`dev` runs against any reachable rockboxd instance — by default Vite proxies
nothing, so point the AWS SDK at a daemon by setting credentials on the login
page; cross-origin requests need the daemon to be on the same origin (or a
local proxy added to `vite.config.ts`).

## Embedding in rockboxd

`crates/s3/src/admin.rs` embeds `s3webui/dist/` with `rust-embed` and mounts
the SPA at `/admin/` (bare `/admin` 301-redirects to `/admin/` so the
relative asset paths resolve and the catch-all `/{bucket}` route doesn't
treat `admin` as a bucket name). Unknown sub-paths fall back to
`index.html` for client-side routing.

After editing the UI, rebuild `dist/` before rebuilding the crate so the new
bundle is picked up by `rust-embed`:

```sh
cd crates/s3/s3webui && bun run build
cargo build --release -p rockbox-server
```

## Auth model

Credentials live only in memory (Jotai store) and are sent on every request
as SigV4 headers — there is no cookie, no session token, and no server-side
auth state. Logging out clears the atom. The login page calls
`HeadBucket music`; the daemon responds 200 on a valid key pair and 403
otherwise.
