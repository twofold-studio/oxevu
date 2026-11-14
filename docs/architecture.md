# Architecture

Status: local server and web client include managed FFmpeg/NVIDIA playback, settings, user access and watch parties. Hardware decoding, HDR conversion and mobile packaging remain pending.

Oxevu is one Rust service with a bundled web UI. FFmpeg and ffprobe are external executable dependencies; the server supervises their jobs. Application data and media remain outside the executable. Mobile apps connect to the same server and do not run a second media server.

## Initial choices

- Rust with Tokio and Axum for HTTP and background work.
- SQLite with SQLx and versioned migrations for users, libraries, catalog and progress. No separate database service.
- Svelte 5 and TypeScript with Vite Plus, shadcn-svelte components and Tailwind CSS v4 for a responsive client. Build static assets and embed them in release binaries. No Node runtime in production.
- Versioned JSON API under `/api/v1`, described with OpenAPI. Generate client types from the API contract.
- One Cargo workspace, initially one server crate. Use modules rather than a crate per feature; extract crates when a real independent boundary appears.
- Structured tracing with request/job IDs. Never log passwords, session tokens or complete media access URLs.

These are project defaults, not claims of validated playback or mobile compatibility.

## Planned source layout

```text
crates/server/
  src/
    main.rs          # configuration, startup and shutdown
    app.rs           # application state and routing
    auth/            # accounts, sessions and authorization
    library/         # roots, scanning, identification and catalog
    playback/        # capability negotiation, sessions and delivery
    jobs/            # bounded task execution and cancellation
    storage/         # SQLite queries and migrations integration
    api/             # HTTP contracts and handlers
    config/          # validated configuration
  migrations/
  tests/             # API and end-to-end server tests
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

## Boundaries and data

HTTP handlers validate input, authorize the caller and invoke domain operations. Domain modules do not depend on HTTP request types. Storage owns queries; playback owns child processes and temporary files. Future requests, management, indexer and torrent modules call explicit internal interfaces rather than making HTTP calls back into this service.

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
