# Changelog

Notable changes to Oxevu, grouped by version. Add ongoing work under Unreleased;
move those entries into a dated version when preparing a release. Record what
changed and any useful decisions or limitations. Only describe implemented work
as available; planned features belong in GOAL.md.

## Unreleased

### Changed

- Catalog, library-root updates, scanning and identification now separate SQL repositories, HTTP adapters and an independent library service crate. Playback fallback and cancellation use typed process errors; acquisition owns workflow coordination, and the torrent test hook is feature-gated.

- REST routes now register their OpenAPI contracts alongside the real handlers. All frontend JSON API calls use a pinned Hey API TypeScript SDK generated from Rust wire types; the generic response-casting client is removed. Production builds regenerate bindings and type-check before bundling, while contract drift checks reject stale artifacts. Native media stays on HTTP; foreign migration rows retain explicit source-specific validation.

- The backend is now a twelve-crate Cargo workspace, separating the server executable and API workflows from library services, configuration, database/migrations, media inspection, acquisition rules, indexer parsing, torrent lifecycle, safe filesystem publication, playback processes/cache and embedded web assets. Libraries take explicit inputs rather than shared application state; migration checksums and service defaults are preserved.

- Series pages are one compact page: the series hero (Play/Resume, Identify, Favorite, Remove, overview) is followed by a full-width Episodes section whose season tabs sit directly above the selected season's summary and episodes. Season covers and separate season pages are replaced by the tab strip; `?season=` links open that tab, and season favorites and removal sit beside the season heading. Continue watching stays beside the title on wide screens. Movie and episode pages share the same hero scale and action sizes; episodes show a Series › Season › Episode trail and full-width previous/next links.

- Player volume feedback appears in the center with smoother seek/volume animations and new play/pause indicators; reduced-motion preferences remain supported.

- Watch parties opens from the top-right header between the account menu and search, replacing its rail and mobile More links.

### Fixed

- Series identification prefers unique episode titles over release numbering, covering combined releases such as House season six whose local episode 22 corresponds to TMDB episode 21.
- Metadata searches preserve numbers in titles such as Blade Runner 2049 and 2001: A Space Odyssey when a later release year is present in the filename.

- The title-page Watch together action appears only for a joined live party and opens playback in that existing party instead of creating another one.

- Wide search-result dialogs allow more horizontal space, keep their grid content within the viewport, and wrap long subtitle release names without pushing Download controls off screen.

- Import previews explain missing torrent identities instead of returning a generic database error. Automatic imports use a default movie/series destination without requiring a per-title Wanted rule.

- Imported titles keep their catalog cover, genres, provider link and rating. Existing imports receive missing cover metadata without replacing owner matches.

### Added

- Automatic metadata enrichment for existing and newly imported library items: read bounded local NFO sidecars, then try configured TMDB/OMDb providers with conservative title/year matching. Unmatched titles show Needs identification and retry after six hours; manual matches take priority. Identify also accepts TMDB IDs/URLs and IMDb IDs/URLs for OMDb.

- Series identification supports up to 2,000 episodes and reuses each TMDB season response instead of requesting every episode separately.

- The flake provides a reproducible server package with embedded web assets and a NixOS service module with persistent state, FFmpeg and read-only media access.

- Accounts can upload and crop avatars with circular previews, zoom and positioning, or choose from 20 presets across abstract, animals, people and space. Avatars are saved per account and shown in the header.

- Title details offer favorites in place of opinions; show and season actions favorite their episodes. Source file stats open from the metadata row beneath the title.

- Subtitle timing controls offer earlier/later adjustments, exact seconds and reset, remembered in this browser per title and track. Personal timing is combined with HLS seek offsets without modifying subtitle files or party playback.

- Library cards, episodes, details and Downloads link to recognized transfers with download status and Play/Resume when streaming is available. Progressive playback shows buffering status, uses a bounded startup wait and retains session leases without rapid heartbeats. Visible library views share a ten-second refresh loop.

- Recognized torrent videos appear as downloading library entries. Playback reads verified pieces with range/seek priorities, supports the existing FFmpeg conversion path through a private loopback reader, respects paused transfers and queue limits, and retains the same item ID and user history after full import. Growing sources are excluded from finalized transcode caching.

- Movie, series and season pages offer confirmed removal, retained-entry badges, personal Like/Dislike controls and links to find releases again. Missing files are excluded from playback and Continue watching.

- Admins can preview and confirm permanent removal of selected movie, season or series files, either keeping metadata/history or forgetting the entries. Shared hard links are reported separately from estimated reclaimed space.
- Retained entries keep watch history, favorites and per-user like/dislike opinions. Rescans preserve them, intentional removals do not automatically reacquire, and matching later imports retain their identity and history.

- Torrents are recognized from movie release names or episode numbering using a unique exact TMDB match, with review states for ambiguous names. Imports wait for completed, verified downloads and retain seeding sources.
- One effective default import folder per media type, with configurable overrides and an authenticated endpoint for manually identifying a torrent.

- Item pages tuck file size, format, resolution, bitrate, codecs and audio channels into a collapsed Stats for nerds section. New scans record these technical facts; older items can be rescanned.

### Removed

- Routine browser codec/conversion notices from the item detail page.

### Decisions

- Direct torrent traffic is permitted; a VPN is optional. Pin librqbit 9.1.0 for
  the embedded-engine prototype, with remaining compatibility gates documented.

- Proposed a phased requests, movie/series management and embedded BitTorrent
  replacement plan, with engine/network validation and staged migration gates.

### Changed

- Download rows use compact status icons, recognition badges and a collapsible transfer-speed control. Imports settings show effective movie/series destinations, and uncertain torrents can be identified from catalog search results.

- The application header uses the existing rust-orange logo artwork; the favicon is unchanged.

- Activity offers persistent per-entry download caps alongside combined transfer
  caps, with autosave and validation. Entry changes reconnect only that torrent.
- Completed downloads explain missing import destinations or title associations,
  show successful imports, and clear stale warnings after import. Import settings
  explain the automatic 10-second check and retained seeding files.

- Release search has cancellation, elapsed time, partial/empty/failure states,
  profile-change warnings, stable result keys and inline download outcomes.
- Activity identifies transfers by release and indexer, shows ETA and job IDs,
  explains provider errors and makes disabled/pending imports visible.
- Profile size controls display GiB, including an unlimited maximum; existing
  profile values and the built-in default are preserved.

- Release search preserves XML entities and full magnet IDs, groups the same
  torrent across sources, and reports named source failures with retry guidance.
  Full indexer pages return bounded results instead of failing.
- Torrent retrieval accepts validated magnet redirects and one same-origin HTTP
  redirect, retains host restrictions, and caches resolved magnets for retry.
- Download responses retain release labels and expose import status. Profile
  evaluation supports an unlimited maximum and reports actual sizes and limits;
  existing profiles and the built-in default are preserved.

- Watch parties can start as empty lobbies and stay joined while browsing.
  Any member can start a title, pause or seek for everyone. Volume, subtitles and
  stream quality remain personal; Back leaves the player without leaving the party.
  This browser remembers its volume, mute and subtitle-language/off preferences
  when changing titles.
- Indexer tests keep a stable button footprint, animate between the connection,
  spinner and result icons, and show inline success or failure without reloading rows.

- Settings open from the account menu and replace the application rail with a
  grouped settings navigation (Media, Playback, Downloads, Sources, People, Your
  account) and Back to library. Each former tab is split into focused,
  deep-linkable pages such as `#/settings/downloads/network`; old tab links
  redirect to the first page the user may open. Viewers see only their account,
  delegated indexer managers see Indexers and their account, and direct links to
  other pages explain the missing permission. Phones get a settings index page
  instead of the bottom navigation. Pages editing the same playback or download
  settings record save only their own fields through one ordered queue.

### Added

- Activity has autosaving global download/upload speed limits for administrators,
  applied to running transfers without restarting the torrent engine.

- Series overviews place Continue watching and season covers beside the title
  and details on wide screens. Viewport-height spacing brings seasons into the
  first screen where space permits; smaller screens retain natural scrolling.
  Season covers include episode counts and watched progress.

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

- Reject new requests already satisfied by the viewer's accessible library,
  while keeping retries of an existing request idempotent.

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
