# Torrent engine acceptance

Use **librqbit 9.1.0** for the initial embedded-engine prototype. It is pinned
exactly, with default features off and Rust TLS enabled. Its standalone HTTP API
and web UI are not enabled. The `torrent-spike` Cargo feature builds a development
example. The application now embeds the same pinned engine through the native
[acquisition adapter](acquisition.md), with authenticated routes and a queue.

The harness evidence below covers milestone 0. Native application checks are
described in the acquisition guide and [acceptance record](acquisition-validation.md),
including a public Ubuntu TCP/uTP transfer. Neither authorizes retiring qBittorrent.
Direct traffic is permitted by the owner; no VPN is required.

## Repeat the checks

On Linux, with the development shell available:

```sh
nix develop -c cargo run --locked -p oxevu-server \
  --example torrent-spike --features torrent-spike
```

The harness generates a 2 MiB payload and a separate unselected file. It starts
an HTTP tracker and seed on ephemeral loopback ports, then launches downloader
processes from the same example executable. All peer sockets bind to `lo`; DHT,
local discovery and UPnP are disabled. Nothing contacts a public swarm or uses
existing media. The example's internal `--download-stage` arguments are only for
its child processes.

The outer run has a 90-second deadline, each downloader stage a 40-second deadline.
Cancellation kills and reaps the downloader before removing its files. Sessions
are stopped, tracker tasks aborted and temporary directories removed on exit.
The example intentionally uses separate processes to test engine persistence;
this does not decide the production process model.

## Observed on vd, 2026-11-16

| Check | Evidence |
| --- | --- |
| Private v1 metadata and HTTP tracker | Private fixture is parsed; tracker discovers the loopback seed, including after restart. |
| Existing-file recheck and seeding | Seed verifies generated files; uploaded-byte accounting increases during the transfer. |
| Selective download | Only `main.bin` receives verified progress; the other file remains unselected. |
| Pause | A partial download stops advancing for the observation interval. |
| Persistent restart | First downloader exits after partial progress; a new process restores it and completes (114,688 verified bytes in the final run). |
| Download limit | 256 KiB/s token bucket permits a partial pause and constrains total transfer time; this is a coarse sanity check. |
| Integrity and retained files | Downloaded payload equals the source byte for byte after job removal. |
| Duplicate prevention | Re-adding the same infohash returns `AlreadyManaged`. |
| Public v1 magnet | Metadata exchange and selected-file transfer complete through an explicit loopback peer; bytes match. |
| Missing interface | Session startup with a nonexistent interface fails. |
| Shutdown | Session TCP listeners reject new connections after stop; downloader processes exit and are reaped. |
| Interruption | SIGINT returns the expected interruption error, with no downloader processes or fixture directories left behind. |

These checks used small generated data, IPv4 and TCP. They do not measure
production memory, disk or hashing contention, sustained speed, or coexistence
with playback. No real queue, media directories, services or network defaults
were changed.

Formatting, strict all-target/all-feature Clippy, default and all-feature Rust
tests (48 passed, four hardware tests ignored), changelog validation and
three-platform Nix evaluation passed. The full development-shell invocation was
stopped while fetching its Android image; the harness was built and run with the
cached Rust/CMake tools instead. No hardware tests were rerun for this change.

## Limits to handle before production integration

- **Private trackers:** the pinned engine truncates private torrents to their
  first tracker. Do not silently claim multi-tracker support. Test the intended
  trackers' announce/passkey behavior and rules before importing their jobs.
- **Private magnets:** the seed refuses metadata exchange for private torrents.
  Obtain their `.torrent` metadata through the configured indexer; the passing
  magnet check uses a separate public fixture.
- **Persistence:** explicit initial peers are not persisted. Restarts need a
  working tracker or supported discovery. The harness waits for initialization
  before unpausing a restored torrent. JSON engine state is distinct from the
  future SQLite job ledger; it is not a qBittorrent fast-resume importer.
- **Shutdown:** `Session::stop` pauses torrents, cancels a token and sleeps one
  second. Listener closure passed, but the API does not join every internal task.
  Stress cancellation and in-flight disk writes before enabling production jobs.
- **Formats:** v1 is verified. Pure v2 is not accepted by this spike; hybrid
  torrents and their v1 fallback need fixtures. No v2 integrity claim is made.
- **Network coverage:** UDP trackers, uTP, IPv6, DHT, PEX, DNS, authenticated
  torrent retrieval and interface disconnect/reconnect remain unverified here.
  Direct routing is allowed; a VPN-only mode would need separate full-path tests.
- **Policy and recovery:** hard crashes, disk exhaustion, malformed metadata,
  path containment, large queues, upload limits and ratio/time rules still need
  acceptance in Oxevu's adapter. Session limits are not a disk-space quota.

## Dependency review

The engine and its rqbit crates declare Apache-2.0. Cargo's resolved manifest
licenses were inspected, including Rust TLS, AWS-LC's native build and
MPL-2.0 `option-ext`. CMake is included in the development shell for the native
TLS dependency. This is a dependency inventory, not a completed distribution
notice audit; Oxevu's own release license and bundled notices remain open.

Pinned upstream source:

- [Engine manifest](https://github.com/ikatson/rqbit/blob/4309d38769ef93f6ee4f01a1ab147009a197fa8f/crates/librqbit/Cargo.toml)
- [Sessions, tracker selection and stop](https://github.com/ikatson/rqbit/blob/4309d38769ef93f6ee4f01a1ab147009a197fa8f/crates/librqbit/src/session.rs)
- [Private-torrent metadata and PEX handling](https://github.com/ikatson/rqbit/blob/4309d38769ef93f6ee4f01a1ab147009a197fa8f/crates/librqbit/src/torrent_state/live/mod.rs)
- [JSON persistence fields](https://github.com/ikatson/rqbit/blob/4309d38769ef93f6ee4f01a1ab147009a197fa8f/crates/librqbit/src/session_persistence/json.rs)

Continue with the [acquisition plan](acquisition-plan.md). Requests can be built
while the remaining compatibility gates stay explicit; automatic real downloads
and migration must wait for their acceptance.
