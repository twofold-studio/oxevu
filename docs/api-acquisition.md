# Acquisition API

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
| `POST /titles/{title-id}/releases` | Download manager; `{season?,episode?,profile_id?,offset?}` |
| `GET /downloads` | Download manager; durable queue, sanitized statistics/files/errors |
| `POST /downloads` | Download manager; one `release_id`, `magnet`, or `torrent_base64` |
| `PUT /downloads/{id}` | Download manager; `{action,selected_files?}` |
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

Import input contains `library_id`, configured `root`, and `mappings`, for example
`[{"file_index":0,"episodes":[{"season":1,"episode":1}]}]`. Movie mappings have
an empty episode list. Preview returns destinations and a token; submit the same
input plus that token to confirm. Changing source identity invalidates it.

Migration input is `{source,records,users,tmdb_ids,profile_id?,token?}`. Source is
`radarr`, `sonarr` or `seerr`. Maps use string external IDs. Preview validates
ownership and required identity mappings without importing. Confirmation resolves
provider titles and reports success/error for each record; resubmission is safe.

See [setup and limits](acquisition.md) for defaults and network/storage rules.
