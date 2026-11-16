# Changelog

Notable changes to Oxevu, grouped by version. Add ongoing work under Unreleased;
move those entries into a dated version when preparing a release. Record what
changed and any useful decisions or limitations. Only describe implemented work
as available; planned features belong in GOAL.md.

## Unreleased

### Decisions

- Direct torrent traffic is permitted; a VPN is optional. Pin librqbit 9.1.0 for
  the embedded-engine prototype, with remaining compatibility gates documented.

- Proposed a phased requests, movie/series management and embedded BitTorrent
  replacement plan, with engine/network validation and staged migration gates.

### Added

- Native acquisition: TMDB discovery, scoped household requests and approvals,
  wanted movies/episodes, multi-user quotas/permissions, Torznab search and feeds,
  acquisition profiles and an embedded BitTorrent queue. Includes selective
  transfers, pause/retry/restart rechecking, speed/storage/seeding limits and
  per-episode reservations across overlapping requests and packs.
- Reviewed and configured automatic library imports with hardlinks or verified
  cross-filesystem copies, no-overwrite publication and recovery journals.
  Added Discover, Requests, Wanted, Activity and Acquisition settings screens.
  Downloads and library writes start disabled; existing services stay in place.
- Previewed Radarr/Sonarr/Seerr export migration with explicit identity, profile
  and requester mappings, plus copy-and-recheck adoption of existing torrent
  data. Originals are retained; imported monitoring starts with automation off.

- Opt-in torrent acceptance harness with generated loopback fixtures: private
  v1 transfer, selective files, pause, persistence across downloader processes,
  seeding accounting, public magnet retrieval, duplicate detection and cleanup.
  Added CMake to the development shell for
  the engine's native TLS dependency.

- Directional seek feedback near the video edges, off-center volume feedback,
  and per-press pulse/ripple animations with a static reduced-motion alternative.

- Simplified player track to elapsed progress and a uniform merged ready-range
  fill; keyboard focus highlights the thumb. Episode navigation sits beside Play.

- Shared overlay rules and centered user-management modals. Task dialogs now
  use common widths, viewport gutters and scrolling on desktop and phones.

- Translucent seek/volume feedback over video and previous/next episode controls
  that follow library ordering across seasons.

- Poster covers in TMDB and OMDb identification results, with a fallback for
  missing or failed images.

- Title, series and season pages show a compact icon-led facts line under the
  title: years, certificate, score with its provider mark, runtime and "Ends at"
  finish time. Credits use icons and genre chips; TMDB/IMDb links use local
  typographic marks. Scores only carry a provider when the record stores one,
  episodes hide unattributed (series-copied) scores, and episode pages label
  the year span as the series'. Finish times refresh every 30 seconds. On phones, episode row
  actions move to their own line so titles and facts are no longer squeezed.

- Metadata refresh preserves local season posters instead of replacing them with
  the series-level provider poster.

- Season cover links open dedicated season views with provider descriptions and
  ratings after identification. Episode rows include descriptions, finish times,
  watched toggles and per-user favorites. TMDB episode scores no longer inherit
  the series score.
- Bottom-toolbar picture-in-picture control and a controls-visible vignette.

- Player timeline displays separate ready ranges from the browser buffer and
  compatible conversion-cache entries when a session opens.

- Separate Resume/Play and episode-information links on the next-up card, with
  a larger play target and reduced-motion-aware hover feedback.

- Configurable arrow-key and skip-button interval in Account settings, defaulting
  to five seconds. Arrow seeking also works when the seek bar or a button has focus.

- Bounded persistent conversion cache with partial-range reuse and automatic
  continuation, configurable in Playback settings (5 GB by default).
- Subtitle-track search for lists longer than eight tracks, with player menus
  kept visible while open.

- Administrator folder-path suggestions while typing library locations, with
  directory-only results and bounded filesystem lookups.

- Playback quality presets and conservative automatic quality reductions for
  limited connections, with server-enforced bitrate and resolution limits.
- Embedded and sidecar text subtitle selection, offset-aware browser captions,
  and optional OpenSubtitles search/download without writing into media folders.
- Administrator integrations for TMDB, OMDb and OpenSubtitles with redacted
  credentials; keyless, attributed studio artwork enabled by default. Explicit
  title identification applies reviewed metadata matches and preserves rescans.

- NVIDIA CUDA decoding/scaling with NVENC or CPU encoding, real capability
  probes, explicit decoder status and tested CPU fallback.
- Automatic saving for playback settings and existing user/library edits, with
  serialized writes, field validation and save/error feedback. Creation, password
  changes and removal retain explicit actions.

- Playback settings UI, multiple-root library management, account password
  changes and administrator user/access management.
- Browser HLS playback with delivery selection, offset-aware seeking, session
  cleanup and absolute progress. Watch-party UI synchronizes host controls and
  handles follower drift and blocked autoplay.

- Backend settings, multiple library roots, users/roles/library grants, and
  independent resume rules, with migration of existing accounts and locations.
- Managed direct/remux/CPU/NVIDIA HLS playback sessions with ownership checks,
  resource bounds, heartbeat expiry, fallback and process cleanup.
- Server-authoritative watch-party routes with host controls and live access checks.
- Read-only Jellyfin Playback/Libraries inventory and playback validation record.

- Shared motion tokens, transition helpers and SwapView for reusable view changes.
- Animated season selection, image reveals and episode status cues with reduced
  motion support. Selected season is retained in the URL.

- Authenticated local posters, backdrops and episode stills, plus cached FFmpeg
  thumbnails. Library cards and details now display available artwork.

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

- Auto quality starts at server limits instead of trusting coarse browser
  bandwidth hints. The menu shows the effective quality and pixel resolution.
- NVIDIA forced IDR keyframes produce actual two-second chunks. Playback also
  reuses produced ranges and buffers toward 90 seconds ahead for quicker seeks.

- Shared segmented controls now clearly distinguish the selected option,
  including library contents and hardware acceleration.

- Provider HTTP requests follow bounded redirects on the original origin, so
  OpenSubtitles search canonicalization works without forwarding credentials
  to another host.

- Scratch disk accounting tolerates FFmpeg atomic file renames, preventing false
  disk-limit failures that interrupted ongoing playback.

- Seeking cancels and reaps starting/active sessions before replacement,
  coalesces rapid requests, and avoids exhausting the transcode job limit.
- HLS uses two-second EVENT segments and starts at the session origin.
- Switches have visible off-state tracks and contrasting thumbs.

- Episode display names omit series prefixes and common release suffixes while
  preserving original filenames and saved progress.

- TV scans group seasons by their show folder when release filenames use
  different title spellings or years.

- Fresh catalog items without progress no longer panic during SQLite decoding.
- Matroska delivery is distinguished from WebM despite ffprobe sharing a format list.

### Changed

- Episodes read as "Season 2, Episode 5" (or "Episode 5" inside a season,
  "Special 3" for specials) instead of S02E05 codes. Show Play/Resume names
  the episode title, and Next up shows "Season 2, Episode 5 - Title".

- Seasons use selectable poster cards with episode counts and labels. Next Up
  captions show cleaned titles.

- Show links use readable title slugs, with redirects from existing ID links.
- Larger detail posters, backdrops and next-episode thumbnails, with responsive
  stacked layouts on phones and larger catalog covers.

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
