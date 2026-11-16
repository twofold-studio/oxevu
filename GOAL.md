# Oxevu goal

Build a simple, self-hosted home for a personal media collection. Oxevu should bring watching, finding, requesting, and downloading media together in one application, replacing the current stack one part at a time.

This is a working plan, not a list of features that exist today. Favor the smallest useful feature set for a home server over matching every option in the applications it replaces.

## Principles

- One Rust server, one shared Svelte interface, and a single understandable setup.
- Make the common path easy: find something, watch it, or request it.
- Reuse proven components where they make sense. A single server executable does not require reimplementing FFmpeg or a torrent protocol from scratch.
- Keep existing media and services safe. Run alongside them until each replacement is proven; never rename, move, or delete files without an explicit action or configured rule.
- Keep modules optional and connected through clear internal interfaces, with shared accounts, settings, and status.
- Start small, measure resource use, and add complexity only when it solves a real need. Do not promise a specific memory reduction before measuring.

## Modules

### 1. Library and player — replaces Jellyfin first

**Goal:** Browse and watch movies and series from existing folders on the web and mobile apps.

- Add movie and series folders; scan them without changing the files. Refresh the library when files are added, changed, or removed.
- Organize movies, shows, seasons, and episodes; search and browse with titles, artwork, and basic metadata. Allow incorrect matches to be corrected.
- Play supported files directly when possible; use FFmpeg to transcode when the client cannot play the source. Support NVIDIA hardware acceleration, with a CPU fallback and a clear indication of the playback method.
- Select audio and subtitle tracks, seek, pause, and resume. Save watch progress per user and show recently watched items.
- Provide a basic admin account, sign-in, and individual profiles with access to the library.

**First useful milestone:** A user can sign in, scan a read-only movie or TV folder, select a video, and play it in a browser; unsupported video can be transcoded. Verify direct play, seeking, resume, and an NVIDIA transcode on `vd` before considering this milestone complete.

**Later:** Multiple users, polished metadata matching, external subtitles, additional devices, and expanded codec support. Native Jellyfin client compatibility, live TV, music, and advanced features are not assumed for the first release.

### 2. Requests — replaces Seerr / Jellyseerr

**Goal:** Let household members find and request a movie or series without needing access to administration screens.

- Search a metadata catalog and show whether a title is already available, requested, or missing.
- Submit a request, view its status, and let an administrator approve or decline it.
- Hand approved requests to the movie or series manager; show failures in plain language.

**First useful milestone:** A request is approved, reaches the appropriate manager, and its status updates when the title enters the library. During migration, adapters may send requests to existing Sonarr and Radarr.

### 3. Movie management — replaces Radarr

**Goal:** Track wanted movies and get them into the movie library.

- Track requested and manually added movies; know whether each movie is missing, downloading, or available.
- Search configured indexers, choose a suitable release using a small set of quality and size preferences, and send it to the download module.
- Import completed downloads into the configured movie folder and tell the library to refresh. Show failed downloads and imports for manual resolution.

**First useful milestone:** One approved movie can be found, downloaded, imported, and played, with no manual file moves. Preserve source files when seeding requires them; support hardlinks where possible and copy when configured or necessary.

### 4. Series management — replaces Sonarr

**Goal:** Track shows and obtain wanted episodes as they become available.

- Track shows, seasons, and episodes; request a show or selected seasons.
- Search for missing episodes, apply basic quality and size preferences, and send selected releases to the download module.
- Import completed episodes into the TV library and refresh the catalog. Display missing episodes and failures.

**First useful milestone:** One requested episode can be found, downloaded, imported, and played. Use the same download, import, and status mechanisms as movies.

### 5. Indexers — replaces Prowlarr

**Goal:** Search configured torrent sources from one place for movies and series.

- Add and test indexers, starting with a small number of compatible sources or standard APIs.
- Search and normalize results into a shared format with title, size, source, and available peer information.
- Send selected results to the download module; surface unavailable sources and errors.
- Keep indexer credentials private and do not include them in logs.

**First useful milestone:** Search one configured indexer and pass a selected result to the download module. No need to replicate Prowlarr's full indexer catalog initially.

### 6. BitTorrent downloads — replaces qBittorrent

**Goal:** Download and seed torrents reliably with a simple queue.

- Accept magnet links and torrent files from other modules or an administrator.
- Show progress, speed, peers, status, and errors; pause, resume, and remove jobs.
- Resume downloads after restart and notify the movie or series manager when files are ready to import.
- Configure download location, basic speed and seeding limits, and network binding. Make the active network path clear; if VPN-only operation is configured, fail closed when that interface is unavailable.
- Removing a job must not delete downloaded files unless the user explicitly selects that action.

**First useful milestone:** Download and resume a known legal test torrent, import its files through the media pipeline, and verify configured network binding. Evaluate a mature Rust-compatible torrent engine before considering a custom implementation.

### 7. Web, Android, and iOS clients

**Goal:** Use the same Oxevu experience across browser and mobile.

- Build a responsive Svelte interface shared between the web and packaged Android/iOS apps. Keep platform-specific code limited to capabilities the web layer cannot provide.
- Connect to an Oxevu server, sign in, browse, search, play, and resume across devices.
- Support device-appropriate playback, subtitles, and background/lock-screen behavior where supported. Test on real devices before claiming compatibility.

**First useful milestone:** The web app plays a library item; Android and iOS packaging and device playback follow as part of the first module, with platform limitations recorded. Offline downloads, casting, and TV-specific clients can come later.

### 8. Shared server and administration

**Goal:** Make the modules work as one service that is easy to install and maintain.

- Provide shared authentication, permissions, configuration, storage, logs, and health/status for all enabled modules.
- Store application data separately from media files. Provide backup and restore for settings, users, and progress.
- Expose a documented local API to the clients; package the built web app with the Rust server where practical.
- Offer explicit settings for paths, transcoding, indexers, and network access. Never expose an unauthenticated admin or playback API by default.
- Report errors and active jobs in the UI. Measure idle and active memory, CPU, and playback performance on `vd`.

**First useful milestone:** Start the server with persistent data, configure a library, authenticate, and view service health. Installation, upgrades, and backups must be documented before replacing existing services.

## Order of work

The next acquisition phase follows the proposed
[replacement plan](docs/acquisition-plan.md): verify the embedded torrent engine
and network policy, then requests, one movie through download/import, TV coverage,
automation and staged migration. It does not authorize retiring the old services.
The [librqbit spike](docs/torrent-engine.md) verifies initial loopback transfers
and process restart; compatibility and production resource checks remain open.
Direct torrent traffic is permitted by the owner; no VPN is required.

The native [acquisition workflow](docs/acquisition.md) now implements requests,
movie/episode monitoring, Torznab search/feeds, embedded transfers, imports and
reviewed export migration. Automatic downloads and imports start disabled.
Representative tracker acceptance and an owner-approved cutover remain separate;
no existing service has been retired.

1. Write the playback specification and define the first end-to-end slice.
2. Build the shared server foundation and the library/player, initially alongside Jellyfin.
3. Deliver and test the web interface, then package and test the shared interface on Android and iOS.
4. Add requests and movie/series management, using existing services as temporary adapters where useful.
5. Add indexer and BitTorrent modules, then replace integrations one at a time.
6. Retire an old service only after its replacement meets the household's needs and its data and workflows have been verified.

The order of the later modules may change as implementation reveals dependencies. All existing services remain available during development.

## Open decisions

- Mobile packaging and playback technology, especially iOS codec and background-playback constraints.
- Metadata providers and their terms, rate limits, and credentials.
- Which existing watch history, users, and library metadata should migrate from Jellyfin.
- Torrent engine acceptance beyond the librqbit spike, indexer coverage and
  optional interface binding.
- Release platforms, license, and exact distribution format.
