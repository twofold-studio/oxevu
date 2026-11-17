# Acquisition replacement plan

Status: native implementation added, 2026-11-16. The [acquisition guide](acquisition.md)
covers requests, wanted units, Torznab, queue administration, imports, automation
and reviewed migration. The [torrent acceptance harness](torrent-engine.md)
pins librqbit 9.1.0. Keep existing services running until representative production
acceptance passes and the owner authorizes retirement.

## Outcome

Oxevu handles discovery, requests, movie/episode monitoring, release search,
BitTorrent transfers and library imports. One Rust executable, the existing
SQLite database and the shared Svelte interface. FFmpeg remains external.
Movie and TV management share a pipeline rather than becoming separate apps.

| Existing application | Oxevu responsibility |
| --- | --- |
| Jellyseerr / Seerr | Discover titles, request movies/seasons/episodes, approvals and status |
| Radarr | Wanted movies, release selection, quality profiles and imports |
| Sonarr | Series/episode catalog, monitoring, air dates, missing episodes and imports |
| qBittorrent | Embedded torrent engine, queue, resume, seeding and network limits |
| Prowlarr | Indexer access: Torznab first; tracker-specific adapters are a later milestone |

Torznab may connect directly to a compatible indexer or temporarily through the
existing Prowlarr instance. That bridge does not constitute replacing Prowlarr.
Sonarr/Radarr/qBittorrent adapters are optional migration tools, not required
runtime dependencies of the completed replacement.

## Milestones

### 0. Verify foundations and choose the torrent engine

- Read-only inventory of current roots, profiles, monitored titles, indexers,
  torrent counts and seeding requirements before proposing migration mappings.
- Evaluate a pinned released `librqbit` as an embedded library, not its standalone
  web server. Check API coverage, licenses/dependencies, private torrents,
  selective files, speed limits, seeding accounting and resource use.
- In a disposable setup, transfer legal fixture media, pause/resume, restart,
  recheck existing files, seed and stop cleanly. Include v1/v2/hybrid support in
  the compatibility matrix; do not promise formats the chosen version lacks.
- Prove the intended network policy covers peers, trackers, DHT/PEX, DNS and
  metadata/torrent retrieval, including IPv6 and disconnect/reconnect failures.
  Separate indexer/provider traffic from peer traffic in the documented policy.
- If library-level binding cannot meet the policy, decide whether to use another
  engine or an isolated worker started from the same executable. Do not route
  playback and metadata through a VPN implicitly or claim VPN safety from a
  socket setting alone. A same-binary worker preserves one executable but adds
  another process and may need host network-namespace configuration.

Exit: recorded engine/version decision and repeatable transfer, restart and
network-isolation acceptance. No real queue import or automatic downloads yet.

Loopback TCP/v1 and a capped Ubuntu public-transfer smoke test pass, including
restart recovery and short playback coexistence. Broad tracker compatibility and
sustained resource acceptance remain gates; see the [acceptance record](acquisition-validation.md).

### 1. Requests, wanted titles and permissions

- Add Discover using the existing TMDB integration. Match availability by stable,
  namespaced provider IDs, not display titles or slugs. Reconcile existing library
  matches; show partial seasons and unknown/offline availability accurately.
- Request a movie, show, selected seasons or episodes. Coalesce shared demand
  without losing individual requesters, approvals or cancellation ownership.
- Admin approval/decline, request limits and optional per-user auto-approval.
  Start with manual approval. Separate `can_request` from permissions to approve,
  manage releases/downloads, configure indexers or select writable destinations.
- Persist the wanted movie/show/episode catalog separately from files actually
  present. Downloading/importing must not mark a request available prematurely.

Exit: two users can request overlapping scopes, an admin can approve them, and
existing media/ACLs prevent duplicate or inaccessible requests. No fake progress.

### 2. One movie through the full native pipeline

- Configure and test one Torznab endpoint; read its capabilities and categories.
  Bound XML size, pagination, concurrency, retries and provider rate limits.
- Normalize candidate IDs, infohash, title, size, seeders, quality, codecs,
  languages and source. Treat unknown facts as unknown. Never log passkeys or
  authenticated download URLs; constrain redirects and untrusted result URLs.
- Add a small acquisition profile: resolution, source type, languages, size
  bounds and allowed codecs/HDR. This is independent of playback bitrate limits.
  Explain every accepted/rejected candidate; manual selection comes first.
- Transfer with the embedded engine. Queue supports pause/resume, retry, limits,
  progress, peers, errors and remove-job-without-deleting-files.
- Preview the import destination, then import only after the configured write
  rule permits it. Validate/probe files; reject traversal, symlinks escaping the
  roots, samples and ambiguous title matches. No archive extraction initially.
- Hardlink on the same filesystem. Cross-filesystem copies require an explicit
  configured fallback, enough reserved space and checksum verification before
  publication. Publish via a same-directory staging file and atomic rename.
  Never overwrite an existing media file or delete the seeding source by default.
- Scan the imported file, refresh metadata, mark available and offer Play.

Exit: one legal movie fixture goes from approved request to playable item with
no manual file movement, while its download remains seedable. Test crashes during
download/import and a retry without duplicate torrents or duplicate imports.

### 3. Series and season workflows

- Persist seasons, episodes and air dates, plus explicit monitoring scopes:
  selected episodes/seasons, all existing episodes, or future episodes.
- Start with ordinary season/episode numbering. Add season packs and multi-episode
  releases with explicit coverage and selective downloading; ambiguous files go
  to manual review. Anime absolute numbering/DVD ordering need separate support.
- Import each mapped episode once. A partially imported season stays partial;
  shared packs can satisfy multiple requests without multiple downloads.
- Provide missing/unaired/available views, manual release search and import review.

Exit: individual episodes and a season pack import correctly, including a failed
or unwanted file, restart recovery, rescans and playback of the imported episodes.

### 4. Scheduled acquisition and useful administration

- Poll configured feeds and schedule bounded missing-title searches. RSS handles
  newly published releases; it does not replace historical searches. Account for
  air dates, time zones, release delays, quotas, failed-candidate blocks and backoff.
  An offline library root suspends replacement decisions; it must not trigger
  duplicate downloads of titles whose availability cannot currently be checked.
- Start without automatic quality upgrades. Add explicit upgrade cutoffs and
  preview/review before any existing file replacement or deletion is enabled.
- Configure upload/download limits, concurrency, disk watermarks, seed ratio/time,
  tracker-specific requirements and pause-on-limit behavior. Do not delete content
  automatically when a seed target is reached.
- Show job history, failure reasons, repair/retry actions and request status in
  the same app. Add in-app notifications first; external notifications can follow.

Exit: unattended new-episode acquisition survives restart and transient failures
without duplicate jobs, excessive searches, disk exhaustion or playback starvation.

### 5. Migration and cutover

- Preview imports of request history, wanted/monitored titles, quality mappings
  and torrent metadata from supported exports/APIs. Preserve accounts/progress.
  Existing torrents need path mapping and content rechecking; do not assume
  another client's fast-resume state is interchangeable.
- Keep one active owner per monitored title/transfer during transition. New
  Oxevu test titles must not also be grabbed by Sonarr/Radarr.
- Audit representative ordinary/private torrents, movie imports, season packs,
  missing mounts, seeding and multi-user access. Record resource use during
  simultaneous streaming, conversion, hashing and transfers on vd.
- Prepare rollback, backups and counts/path comparisons. Stop or remove old
  services and their data only after a concrete review and explicit permission.

Exit: retire each application independently after its acceptance checklist passes.
Usenet, broad tracker scraping, plugins, advanced custom-format languages and
streaming incomplete downloads are later work, not hidden requirements of v1.

## Data and implementation boundaries

Use modules under `crates/api/src/acquisition/`: `requests`, `catalog`,
`indexers`, `releases`, `downloads`, `imports` and `jobs`. Create directories only
as code arrives. Keep movie/episode policies small and share the durable pipeline.
Frontend feature directories cover Discover, Requests, Activity and administration;
use the existing metadata cards, status components and overlay rules.

Persist requests/requesters, monitored titles/episode coverage, profiles, indexers,
release candidates, download jobs, import operations and bounded event history.
Use provider IDs for identity, infohash for torrent deduplication, and explicit
download-file-to-episode mappings. A torrent is not a title: packs are many-to-many.

Keep independent lifecycles: request approval, transfer, import and availability.
Import can complete while seeding continues. Job rows store intent, checkpoints,
attempts and next retry; worker leases and idempotency keys prevent duplicate work.
Reconcile with the engine and filesystem after restart. Journal file publication
so a crash between file placement and SQLite commit can be recovered safely.
No network request or large copy inside a long SQLite transaction.

Scans remain read-only. Only the importer writes to explicitly enabled managed
roots; viewers supply IDs, never arbitrary paths. Separate downloads, library
destinations, app state and transcode cache. Reserve disk space across all of them.
Bound hashing, imports, metadata requests and torrents; prioritize active playback.
Use the same authenticated API and server-side ACL checks for every new resource.

## Network decision

The owner permits direct torrent traffic (2026-11-16). A VPN interface or network
namespace is not required for real downloads. Peer, tracker and discovery traffic
may use the server's normal routes. Indexer and metadata requests keep their own
ordinary routing; no global proxy or playback routing change is implied.

The acceptance harness remains loopback-only, with DHT, local discovery and UPnP
disabled. Optional interface binding needs its own acceptance before being
offered as a VPN-only setting; these loopback checks do not establish VPN safety.

## Decisions before real downloads

1. Initial indexers: which are already Torznab-compatible, and is a temporary
   Prowlarr bridge acceptable for sources needing it?
2. Download/staging roots and approved writable library destinations. Are they
   on the same filesystem? Is copy fallback acceptable?
3. Initial acquisition profile, languages, seeding requirements and which users
   may auto-approve. Proposed rollout: admin approval and manual release choice,
   no upgrades or automatic deletion, then opt into automation after acceptance.

Do not change live defaults, paths or exposure merely by accepting this plan.
The first implementation step is milestone 0, then the requests/catalog slice.

## Sources checked for planning

- [rqbit](https://github.com/ikatson/rqbit) describes embedded-library use and its
  torrent capabilities; exact released-version behavior still needs the spike.
- [librqbit manifest](https://github.com/ikatson/rqbit/blob/main/crates/librqbit/Cargo.toml)
  declares Apache-2.0 and Tokio integration; verify the selected release and its
  dependency licenses before adoption.
- [Torznab specification](https://torznab.github.io/spec-1.3-draft/torznab/Specification-v1.3.html)
  defines capabilities, categories and generic/movie/TV search requests.
- [Servarr storage guide](https://wiki.servarr.com/docker-guide) explains shared
  filesystem layouts for hardlinks; inspect the actual mounts before choosing roots.
