# Acquisition API

## Automatic recognition and import defaults

Acquisition settings additionally accept `auto_identify` (default true),
`movie_destination` and `series_destination`. A destination is
`{library_id,root}` or null; the server records its filesystem device when saved.
Null selects the first enabled matching library/root, in creation order.
`GET /api/v1/acquisition/import-defaults` returns the effective `movies` and `tv`
destinations. Explicit per-title destinations take precedence. Disabled or removed
explicit defaults require choosing a new default; offline roots suspend imports.

The 10-second import worker attempts recognition when torrent file metadata becomes
available. Movie release names and `SxxExx`/season names select the search kind;
only a single exact normalized title match with the same release year, when known,
is accepted. Mixed-title packs and uncertain matches remain for review. Provider
failures and unresolved names retry after five minutes. This does not enable
automatic release searching or change download selection.

Downloads include `recognition` (`pending`, `matched`, `needs_review`, or
`unavailable`, plus a message) and `import_destination`. Administrators or users
allowed to manage downloads can explicitly associate a torrent with
`PUT /api/v1/acquisition/downloads/{id}/identity`, body
`{kind:"movie"|"tv",provider_id:number}`. An active or completed import prevents
reassociation. Missing identities in manual import previews produce a helpful
error rather than a database error.

With library writes enabled, completed verified standalone downloads also import
without a Wanted rule. Unwanted samples/extras are skipped. Ambiguous episode
numbers or multiple main movie files require review. Existing media is never
overwritten, the seeding source remains, and imports honor configured copy fallback.

All routes are under `/api/v1/acquisition`, use the existing HTTP-only session,
and require the configured same-origin `Origin` on mutations. JSON bodies reject
unknown fields. Admins have every acquisition permission; delegated permissions
are separate from library/playback access.

| Route | Permission / purpose |
| --- | --- |
| `GET /access` | Signed-in user's acquisition permissions |
| `PUT /access/{user}` | Admin; request, auto-approve, request/download/indexer management and quota |
| `GET /discover?query=&kind=movie&page=1` | Signed in; TMDB search/popular, pages 1–20 |
| `GET /titles/{movie\|tv}/{tmdb-id}` | Signed in; cached title, seasons, episode catalog and accessible availability |
| `GET /artwork/{file}` | Signed in; bounded TMDB JPEG proxy |
| `GET /requests` | Own requests; approvers see all |
| `POST /requests` | Request permission; `{kind,provider_id,scope}` |
| `PUT /requests/{id}` | Owner cancellation or approver decision `{status,note}` |
| `GET /monitored` | Request/download managers; wanted units, dates, availability and destination rules |
| `PUT /monitored/{title-id}` | Admin; monitor scope, destination, profile and automation |
| `GET /release-sources` | Download manager; enabled source IDs/names, without configuration or secrets |
| `POST /titles/{title-id}/releases` | Download manager; `{season?,episode?,profile_id?,offset?,indexer_ids?}` |
| `GET /downloads` | Download manager; durable queue, sanitized statistics/files/errors |
| `POST /downloads` | Download manager; one `release_id`, `magnet`, or `torrent_base64` |
| `GET /downloads/{id}` | Download manager; current job outcome, including asynchronous startup failure |
| `PUT /downloads/{id}` | Download manager; `{action,selected_files?}` |
| `PUT /downloads/{id}/limits` | Download manager; `{download_limit_kib}`; zero removes the entry cap |
| `POST /downloads/{id}/imports/preview` | Download manager; inspect mapped destinations |
| `POST /downloads/{id}/imports` | Admin; confirm exact import plan token |
| `POST /downloads/{id}/adoption/preview` | Admin; inspect existing data `{source_root}` |
| `POST /downloads/{id}/adoption` | Admin; confirm copy/recheck migration `{source_root,token}` |
| `GET/PUT /settings` | Admin; storage, engine, bounds, seeding, imports and search interval |
| `GET /profiles`, `PUT /profiles/{id}` | Read: download manager. Write: admin |
| `GET /indexers`, `PUT /indexers/{id}`, `POST /indexers/{id}/test` | Indexer manager; keys are write-only |
| `POST /migration/preview`, `POST /migration` | Admin; reviewed API-export import, with explicit user/TMDB/profile mappings |
| `GET /events` | Download manager; recent bounded in-app history |

Title IDs are namespaced, e.g. `tmdb:tv:94605`. URL-encode path IDs. Library
availability uses stable provider identity and the current user's grants.

```json
{"mode":"seasons","seasons":[1,2],"episodes":[],"cutoff":null}
```

Scopes are `all`, `existing`, `future`, `seasons` or `episodes`. Movie scope is
`all`. Only monitor rules also accept `requests`, the union of approved demand.
Existing/future cutoffs are assigned by the server and retained during ordinary
monitor edits. Specials require explicit selection.

Requests have independent `pending/approved/fulfilled/declined/cancelled` states
and a human-readable `stage`. Transfer/import activity does not itself fulfill
a request. The files must enter a library accessible to its owner. Each owner
can cancel their request without cancelling another person's demand.

Download options include `paused`, `title_id`, `selected_files`, `profile_id`,
`override_profile` and `only_wanted`. `only_wanted` reserves missing units and
requires release identity or uploaded torrent metadata. Automatic selection
refuses unknown/ambiguous matches. Manual profile overrides are explicit.

Actions are `pause`, `resume`, `retry`, `select`, `remove`. Selection pauses the
job; resume is explicit. Retry initializes/rechecks the torrent afresh. Remove
detaches the job without deleting data. A paused uploaded `.torrent` can adopt
existing files before it ever contacts peers.

Release search returns `{releases,errors,searched,offset}`. Errors include source
ID/name and `retryable`. `searched` includes hit count, elapsed milliseconds and
per-source counts. Each result has a stable per-source ID; matching infohashes
are grouped with `also_on` source IDs/names. Raw source URLs/credentials stay on
the server. Existing cached release rows and download references are retained;
new searches use the corrected identity rules without a one-off cache purge.

Searches use at most 20 sources and three concurrent source HTTP requests.
Each source search has a 70-second deadline; torrent retrieval has one 30-second
deadline across semaphore wait and redirects. Torrent retrieval accepts validated
v1/hybrid magnets or one same-origin HTTP redirect, preserving host restrictions.

Download list responses include `import_enabled`. Jobs expose snapshotted
`release_title`, `indexer_name`, `release_size`, and whether any file was imported.
Profile `min_size_mb`/`max_size_mb` remain MiB; zero maximum means unlimited.

Jobs also expose `download_limit_kib` (default zero) and `import_blocker` for a
missing title, destination or enabled monitor. Entry limits persist across
restarts and are enforced alongside the shared session limit, not instead of it.
Rates are whole KiB/s: zero or 16–1000000. Updating a running entry reconnects and
rechecks that torrent without deleting files; paused jobs remain paused.

Import input contains `library_id`, configured `root`, and `mappings`, for example
`[{"file_index":0,"episodes":[{"season":1,"episode":1}]}]`. Movie mappings have
an empty episode list. Preview returns destinations and a token; submit the same
input plus that token to confirm. Changing source identity invalidates it.

Migration input is `{source,records,users,tmdb_ids,profile_id?,token?}`. Source is
`radarr`, `sonarr` or `seerr`. Maps use string external IDs. Preview validates
ownership and required identity mappings without importing. Confirmation resolves
provider titles and reports success/error for each record; resubmission is safe.

See [setup and limits](acquisition.md) for defaults and network/storage rules.
