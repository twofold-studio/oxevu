# Changelog

Notable changes to Oxevu, grouped by version. Add ongoing work under Unreleased;
move those entries into a dated version when preparing a release. Record what
changed and any useful decisions or limitations. Only describe implemented work
as available; planned features belong in GOAL.md.

## Unreleased

### Added

- First Rust server and embedded Svelte web client: administrator setup/sign-in,
  read-only movie/TV scanning, catalog browsing, original-file playback with byte
  ranges, and saved progress. Includes dark responsive library and player views.
- Local development samples copied from Jellyfin, outside the repository.


- Glossary defining shared media, playback, and acquisition terms.

- VERSION file and changelog validation/release commands. Release preparation
  dates the Unreleased notes and updates VERSION without committing or tagging.
- Agent guidance for project structure, implementation order, focused commits,
  validation, and keeping source files small and responsibilities clear.
- Python in the development shell for the changelog helper.

### Fixed

- Fresh catalog items without progress no longer panic during SQLite decoding.
- Matroska delivery is distinguished from WebM despite ffprobe sharing a format list.

### Changed

- Initialized Gitadel development workflow with checked commits and pushes for
  completed steps.

- Shortened AGENTS.md into compact rules, commands, and commit examples.

## 0.1.0 — 2026-11-14

Initial project foundation. There is no runnable media server or client yet.

### Added

- Project goal and staged plan to replace Jellyfin, Seerr, Radarr, Sonarr,
  Prowlarr, and qBittorrent with one Rust service and shared clients.
- Architecture, planned source layout, and development rules. Initial server
  choices are Axum, Tokio, SQLite, and SQLx, with embedded web assets and external
  FFmpeg/ffprobe processes.
- First playback specification covering authentication, read-only scanning,
  direct play, seeking, saved progress, transcoding, and acceptance checks on vd.
- Pinned Nix development shell and direnv setup for Rust, web tooling, FFmpeg,
  SQLite, Java, and Android SDK/emulator development. Flake evaluation passes on
  x86_64 Linux, ARM Linux, and Apple silicon macOS; emulator boot and device
  playback remain unverified. iOS Simulator requires host-installed Xcode on macOS.
- Gitadel UI reference screenshots and design notes for navigation, settings,
  controls, colors, and dark mode.
- Project logo and introductory README.
- This changelog, starting at version 0.1.0.

### Decisions

- Use Vite Plus, Svelte 5, TypeScript, shadcn-svelte, and Tailwind CSS v4 for
  the shared interface, with dark mode as the default.
- Build browser playback first and choose mobile packaging after a playback
  prototype on Android and iOS.
- Keep existing media services running during development and treat library
  media as read-only unless an explicit file operation is authorized.
