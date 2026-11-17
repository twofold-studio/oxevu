# Oxevu

One Rust media server with an embedded Svelte UI. Start with library/playback
alongside Jellyfin; replace the other services later. The first local server and web client are implemented.

Read [goal](GOAL.md), [architecture](docs/architecture.md),
[playback](docs/playback.md), [development](docs/development.md) and
[glossary](docs/glossary.md) as needed.

## Stack and layout

- Rust / Tokio / Axum / SQLite / SQLx across `crates/`: executable in `server`, HTTP adapters and coordination in `api`, library services in `library`, repositories and migrations in `db`.
- Vite Plus / Svelte 5 / TypeScript / shadcn-svelte / Tailwind v4 in `apps/web`.
- Embed release web assets; supervise external FFmpeg/ffprobe. No Node server.
- Use dark mode, semantic tokens and `ft-ui-style`.
- Follow [overlay rules](docs/ui-overlays.md): centered modals for tasks,
  anchored menus for short choices; no application drawers.
- Keep HTTP handlers thin; share API contracts with TypeScript.
- Add directories when needed. Mobile packaging follows a device playback spike.

## Code

- One responsibility per file. Aim below 300 lines; review splitting around 400.
  Functions around 50 lines deserve a second look. These are prompts, not quotas;
  generated code, lockfiles and migrations are exempt.
- Split by purpose: `playback/negotiation.rs`, `library/scanner.rs`, `PlayerControls.svelte`.
- Prefer direct code. Extract shared behavior when it exists; skip speculative
  abstractions, catch-all helpers and components for trivial wrappers.
- Keep API/domain logic outside view markup; prefer derived state to sync effects.
- Use typed errors, bounded jobs and direct subprocess arguments. No production
  unwrap/expect, shell interpolation or blocking work on the Tokio executor.
- Keep edits focused. Explain decisions in comments and substantial changes in docs.

## Invariants

- Media stays read-only unless an explicit action/configured rule permits writes.
- Runtime data and scratch stay outside media/source roots. Cancel/reap processes
  and clean scratch on shutdown; bound concurrency and disk use.
- Authenticate APIs and authorize every resource, including playback segments.
  Never accept arbitrary filesystem paths in URLs. Use HTTP-only web sessions.
- Keep credentials out of logs/bundles/commits and existing services available.

## Checks

For routine fixes, the owner prefers targeted checks over full validation suites,
and no post-deployment live UI/playback checks unless requested. Run the minimum
compilation or focused checks warranted by the change, confirm service/deployment
status, and report what actually ran. Track elapsed time from inspection through
deployment confirmation, with diagnosis, implementation, checks and delivery phases.

Enter `nix develop` or use direnv. Run checks for what changed:

```sh
just changelog-check
nix flake check path:. --no-build --all-systems # Nix changes
# Once server code exists:
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test --locked
```

For web changes, run declared type-check/lint/build scripts and Svelte tooling.
Test authorization, ranges, progress, rescans and process cleanup with legal fixtures.
Milestone acceptance needs direct play, seeking, resume and NVIDIA transcoding on
`vd`. Mobile support needs real-device evidence. Report checks actually run.

## Changes and commits

- Preserve existing work; inspect `git status` before editing.
- Add meaningful notes under `Unreleased` in CHANGELOG.md with the change.
  Plans go in GOAL.md; new domain terms go in docs/glossary.md.
- VERSION is authoritative. Release only when requested: `just release 0.1.1`.
  It prepares notes/version only. Use patch for fixes, minor for additions;
  document breaking changes (normally a minor bump during 0.x).
- Commit and push completed development steps after relevant checks pass.
  Stage intended files and review the diff. Include relevant
  tests, notes and lockfiles. Examples:

```text
feat(library): add read-only scanning
fix(player): preserve seek position
docs: define playback acceptance
chore(dev): add Android tooling
```

Keep future manifest versions in sync on release. Use Gitadel for hosting.
Normal development commits and pushes to the private Gitadel repository are
authorized. Tag releases, publish publicly, rewrite history or discard others'
work only when explicitly requested. Handoff: changes, checks, remaining limits; SHA if committed.
