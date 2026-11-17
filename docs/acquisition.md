# Acquisition

Oxevu has native discovery, requests, movie/episode monitoring, Torznab search,
BitTorrent transfers and library imports. The engine is embedded librqbit 9.1.0.
Sonarr, Radarr and qBittorrent are not runtime dependencies of these workflows.
Tracker-specific scraping remains outside this release; a Torznab bridge may
still be needed for sources that do not expose that protocol themselves.

## Start here

1. Configure TMDB in **Settings → Media → Metadata**. Discover uses Oxevu's own
   credentials, independently of Jellyfin.
2. Add movie/TV libraries with their root folders. Grant viewers access to the
   destination library; downloading something does not grant permission to play it.
3. Open **Settings → Downloads → Engine and storage** and choose a separate
   download folder. Add and test a Torznab endpoint under **Sources → Indexers**,
   then review **Sources → Release profiles** and the other Downloads pages.
4. Enable downloads. Library imports have a separate switch. Both start off.
5. Open a title in **Discover**, click **Find**, then **Download** beside a release.
   Administrators and users with download access search immediately using the
   default profile. A series also offers Find beside each season. Requests and
   monitoring are not required for this manual flow; completed files use the
   configured default destination when library imports are enabled.
6. Accounts without download access can still request titles for approval.
   Administrators manage those under **Requests** and **Wanted**, where they can
   configure a monitoring scope, destination, profile and automatic acquisition.
7. Follow transfers in **Activity**. Inspect/select files, pause, retry, or review
   an import. Imports refresh the library and update accessible requests.

Automatic acquisition combines 15-minute Torznab feed polling and scheduled
missing-title searches. It respects provider air dates, selected scope, library
mounts, failed-release blocks and backoff. Catalog units are reserved atomically,
so overlapping requests/packs share demand while seeding an old episode does not
block acquiring the next one. There are no automatic quality upgrades.

## Release search and transfers

Discover can browse by release year, genre, minimum rating, original language and
sort order. Rating filtering and highest-rated sorting require 100 votes. Filters
stay in the URL across pagination and Back navigation. TMDB title search supports
year only; clear the search to use the other browse filters.

Release search keeps previous results visible during a refresh and can be
cancelled. Source chips show hit counts or named failures; expanding a failed
source shows its error and retry guidance. The summary distinguishes unique
torrents from source hits. Matching infohashes are grouped with “Also on”.
Changing profile requires another search before downloading from the old list.
Review releases show their reasons and require explicit confirmation.

Download outcomes stay beside each release, including immediate worker failures.
Activity keeps the release name, indexer, job ID and import status. Completed
transfers stay outside the library while automatic imports are off; administrators
can review an import explicitly. Download completion alone is not library availability.

Activity has a combined download/upload cap and a download cap for each entry.
Both apply together; zero removes that cap. Enter whole KiB/s, at least 16 for a
nonzero cap. Values save automatically. Changing the combined cap is live;
changing an entry cap briefly reconnects and rechecks that torrent, keeping files
and paused intent. Limits survive a server restart.

With automatic imports enabled, the worker checks verified files every 10 seconds.
The title needs an enabled destination rule in Wanted and missing requested media
that matches the selected files. Activity explains missing prerequisites and
import errors, then shows when media has entered the library. Imports retain the
seeding source and never overwrite existing media. Enable cross-filesystem copy
fallback when the download folder and library cannot share hardlinks.

Profile minimum/maximum sizes display in GiB; the API retains its historical
`*_size_mb` names, measured in MiB. Zero maximum means unlimited and does not
disable the minimum. No existing profile is changed by this upgrade.

## Storage and writes

Default downloads are `<data>/acquisition/downloads/<job-id>/`. Keep this tree
separate from media, source code, conversion scratch and artwork caches. The app
validates canonical paths and rejects overlapping roots.

Scans stay read-only. Only imports write to configured, enabled library roots.
The destination preview includes every source, final path and size. Confirmation
binds the current plan, source identity and mount. Files must be fully verified
by the torrent engine and pass a bounded ffprobe inspection.

Publication uses a same-directory staging file and an atomic no-replace link.
Existing media is never overwritten. Imports prefer hardlinks; crossing a
filesystem boundary requires the copy-fallback setting, free space and a verified
SHA-256 copy. Downloaded sources remain for seeding. A journal recovers interrupted
publication/scanning without adding another import row. A failed or ambiguous
mapping requires review. Archives and samples are not imported automatically.

TV filenames use ordinary season/episode numbering. Season packs select wanted
files; explicit mappings support up to 20 consecutive episodes in one file.
Absolute anime numbering and DVD order need manual mapping.

## Limits and defaults

| Setting | Default / behavior |
| --- | --- |
| Downloads / library imports | Both disabled |
| Incoming port | 6882, separate from the existing client's 6881; zero selects a port |
| Network | Direct traffic permitted; optional device binding, UPnP off |
| Discovery | Public DHT on when the engine is running; local discovery off |
| Active downloads | 2, configurable 1–16; completed torrents may keep seeding |
| Global speeds | Unlimited; nonzero limits at least 16 KiB/s |
| Download storage | 100 GiB ceiling, 10 GiB free-space reserve |
| Seeding | Ratio 1 and 72 active hours; both enabled targets must be reached |
| Zero seed target | Disables that target; both zero means indefinite seeding |
| Search interval | 60 minutes per automatic title; failures back off |
| Default profile | 720–1080p, H264/HEVC/AV1, at least one known seeder, ≤20 GiB, HDR off |
| Request quota | 10 active requests per user; approval required by default |

Storage usage is sampled every four seconds; a short overshoot is possible.
One import/adoption runs at a time. Metadata discovery has a 60-second deadline;
indexer requests have size/concurrency limits. Queue capacity is 1000 jobs and
each torrent is limited to 10,000 paths and the configured size ceiling. Seeding
targets pause a job and retain its files. Removing a job also keeps all files.

Interface binding alone is not a VPN kill switch. Peer, tracker, DHT, DNS and
indexer/provider traffic may use normal routes under the owner's direct-traffic
policy. Do not advertise VPN-only protection from this setting.

Private v1 torrents with one tracker are supported. The pinned engine's private
multi-tracker limitation is surfaced as an error. Private magnets and pure v2
torrents are unsupported. Public/hybrid compatibility beyond the recorded
[engine checks](torrent-engine.md) still needs representative acceptance.

## Permissions and credentials

Viewers can discover and request by default. Admins can separately delegate
request approval, download management and indexer configuration, set quotas, or
enable auto-approval. Destination rules, profiles, migration and filesystem
imports remain administrator actions. Playback/library ACLs remain independent.

Indexer API keys and authenticated release URLs remain on the server. Responses
only report whether a key is configured. SQLite now contains acquisition secrets;
its permissions are 0600. Protect the whole data directory and backups.

## Migrate alongside the old services

**Settings → Sources → Migration** accepts JSON API exports:

- Radarr `/api/v3/movie` arrays.
- Sonarr `/api/v3/series` arrays. Records without TMDB IDs require an explicit
  record-ID → TMDB-ID mapping; Oxevu does not guess from titles or TVDB IDs.
- Seerr/Jellyseerr `/api/v1/request` results. Export every page. Map each old
  requester ID to an existing Oxevu user ID; passwords/accounts are not copied.

Use batches up to 1000 records / 1.5 MB. Preview the exact input and mappings,
then confirm. Select an Oxevu quality profile for each batch; custom formats are
not translated. Imported request approval states are retained, but availability
comes from the accessible Oxevu library. Existing monitors are preserved. New
monitors have automatic acquisition off and need a destination. Imports are
idempotent per source type and external record ID, for one source instance of
each type. Partial batches report errors and can be retried.

For torrents, export original `.torrent` metadata from the old client. Add it in
Activity with **Start paused**, then open its files and **Adopt existing data**.
Preview the old download folder mapping. Oxevu copies same-size existing files,
verifies each copy, retains the originals and stays paused. It deliberately
avoids hardlinks here because the torrent engine may rewrite corrupt pieces.
Resume explicitly to hash every piece; missing/corrupt data can then download.
Old clients' fast-resume counters and uploaded totals are not imported.

Keep only one owner downloading each title/transfer during migration. Confirm
counts, paths, ordinary/private torrents, season packs, seeding requirements and
playback under load before retiring anything. No old-service cutover is automatic.

## Backup and rollback

Before upgrading, stop writes or use SQLite's `.backup` command for a consistent
copy. Back up the server executable, database, integration settings and selected
download directory. Do not copy a live SQLite file without its journal state.

To roll back, stop Oxevu, restore the previous executable and its matching database
backup, then restart. Keep newly acquired downloads/media for review; rollback
does not delete them. Existing Sonarr/Radarr/qBittorrent remain usable. Re-enable
their ownership only after disabling corresponding Oxevu automation/jobs.

## Verification

```sh
cargo test --locked acquisition
cargo test --locked
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cd apps/web
pnpm check && pnpm lint && pnpm test && pnpm fmt:check && pnpm build
```

The acquisition tests generate legal video and use loopback tracker/Torznab
fixtures. They exercise movie and shared-season imports, piece rechecking,
ownership, quotas, storage separation, non-overwrite publication, offline roots,
feed/search deduplication, copied torrent adoption and migration replay. They do
not prove sustained public tracker performance or real-device behavior.

API reference: [acquisition endpoints](api-acquisition.md).
Observed results: [acceptance record](acquisition-validation.md).
