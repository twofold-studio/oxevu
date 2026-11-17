# Architecture

Status: local server and web client include managed FFmpeg/NVIDIA playback, settings, user access and watch parties. Hardware decoding, HDR conversion and mobile packaging remain pending.

Oxevu is one Rust service with a bundled web UI. FFmpeg and ffprobe are external executable dependencies; the server supervises their jobs. Application data and media remain outside the executable. Mobile apps connect to the same server and do not run a second media server.

## Initial choices

- Rust with Tokio and Axum for HTTP and background work.
- SQLite with SQLx and versioned migrations for users, libraries, catalog and progress. No separate database service.
- Svelte 5 and TypeScript with Vite Plus, shadcn-svelte components and Tailwind CSS v4 for a responsive client. Build static assets and embed them in release binaries. No Node runtime in production.
- Versioned JSON API under `/api/v1`, described with OpenAPI. Generate client types from the API contract.
- One Cargo workspace with a thin executable and dependency-directed libraries. Rules, encoding, engine lifecycle, filesystem publication, configuration, database initialization, media inspection and embedded assets are independent of application state. The API crate owns HTTP authorization, persisted workflows and coordination between those libraries.
- Structured tracing with request/job IDs. Never log passwords, session tokens or complete media access URLs.

These are project defaults, not claims of validated playback or mobile compatibility.

## Source layout

```text
crates/server/        # oxevu-server: Tokio entry point and optional torrent harness
crates/api/           # oxevu-api: application lifecycle, HTTP authorization and workflow orchestration
  src/routes.rs      # HTTP composition and origin middleware
  src/acquisition/   # requests, indexers, transfers and import supervision
  src/playback/      # sessions, FFmpeg supervision and delivery
  src/parties/       # watch parties
crates/config/       # oxevu-config: serialized settings, defaults and validation
crates/acquisition/  # oxevu-acquisition: request scopes and release matching rules
crates/indexers/     # oxevu-indexers: bounded Torznab parsing and safe HTTP redirects
crates/transfers/    # oxevu-transfers: torrent engine lifecycle, validation and identity
crates/filesystem/   # oxevu-filesystem: safe publication, checksums and disk accounting
crates/playback/     # oxevu-playback: encoding commands, child supervision and HLS cache manifests
crates/db/           # oxevu-db: SQLite initialization and embedded migration history
  migrations/        # original numbered migrations, contents unchanged
crates/media/        # oxevu-media: bounded ffprobe and episode title parsing
crates/web/          # oxevu-web: web asset embedding and serving
apps/web/
  src/lib/api/       # generated types and request client
  src/lib/components/
  src/lib/features/  # sign-in, library, player and settings
apps/mobile/         # reserved until packaging spike is complete
fixtures/            # small legal media fixtures and their provenance
scripts/             # build, packaging and development helpers
docs/               # architecture, playback and operational guides
```

Create directories when they contain working code. The mobile folder is reserved, not a commitment to a wrapper framework.

`oxevu-server` depends on `oxevu-api`, which composes the lower-level libraries.
No library depends back on the API. Playback depends on configuration and
filesystem primitives; the other lower-level crates have no workspace dependency.
`oxevu-config` and `oxevu-media` expose typed errors; API adapters translate them
to the existing HTTP responses. The database crate owns one migration history,
shared by production startup and isolated tests. Moving migrations must never
change their SQL or checksums.

The process supervisor accepts a session observer for PID updates, lease checks
and live authorization. It owns child creation, cancellation, reaping, output
bounds, startup timeouts and disk limits; it never reads the application database.
Filesystem publication accepts an immutable approved plan and a cancellation
callback, never application state or a torrent runtime. The transfer engine
accepts only network/storage options, not request policy. This keeps shared
application state out of reusable libraries and avoids feature dependency cycles.

Workspace versions, editions and dependency versions are declared in the root
Cargo manifest. Existing module tests run with `cargo test --workspace --locked`;
the torrent harness remains available through `oxevu-server`'s `torrent-spike`
feature. One binary, database, service port and deployment remain in use.

## Boundaries and data

HTTP handlers validate input, authorize the caller and invoke domain operations.
Storage owns queries; playback owns child processes and temporary files.
`acquisition/` shares durable requests, wanted units, profiles, indexers, transfers
and import journals. Workers call internal operations rather than HTTP routes.
SQLite owns transfer intent; engine instances are recreated paused and recheck
existing pieces. See [acquisition](acquisition.md) for storage and write rules.

Persist users, sessions, library roots, items, media files, tracks, scan jobs and per-user progress. Keep stable item IDs distinct from paths. Probe files with ffprobe. Missing or disconnected roots must not cause a whole library to be treated as deleted. Store database, artwork cache and transcode scratch in separate configured paths.

Scans are read-only. Never rename, move or delete media during identification or playback. Manual corrections must survive rescans. Bound scan concurrency, subprocesses and transcode disk usage. Shutdown cancels jobs and reaps children.

## Visual direction

Dark mode is the default, as requested. Use the Gitadel reference study in `docs/design-reference/gitadel/STUDY.md`: compact navigation, restrained neutral surfaces, subtle separators, and artwork-led media views.

## Mobile decision gate

Build the responsive web player first. Then compare a thin web wrapper (Capacitor or Tauri) with a native playback bridge on Android and iOS. Test seeking, track selection, subtitles, resume, server connection, background behavior and HTTPS on real devices. Choose the wrapper after this spike. Reusing the Svelte UI does not guarantee that a WebView player meets playback requirements.

## Decisions still needed

- Available Mac and physical iOS/Android devices for development and acceptance testing.
- Metadata provider and credentials; local filename/probe metadata can start without one.
- Required first-release codecs, subtitle formats and simultaneous streams.
- Access model: local network only initially, or remote HTTPS access immediately.
- Jellyfin history migration and native Jellyfin-client compatibility (deferred by default).
- License and supported release platforms.

Implement the first local web slice while these are resolved. Do not block scanning and direct playback on artwork or mobile packaging.
