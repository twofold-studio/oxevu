# Playback during download

Recognized transfers with selected, unambiguous video files get stable library
items once torrent metadata and a matching default import library are available.
Movies need one main video; TV files need known SxxExx coverage. Samples and extras
are excluded. Recognition and projection run in the existing ten-second worker.
No library media file is written until the existing verified import runs.

## Catalog

`GET /api/v1/items` and `/items/{id}` add an optional `download` object to an
item that is still backed by a torrent:

```json
{
  "id": "download-job-id",
  "file_index": 0,
  "state": "downloading",
  "bytes_received": 32768,
  "size_bytes": 1048576,
  "playable": true
}
```

`file_available` remains false until a verified file is imported. Clients should
distinguish `file_available !== false` (local file) from
`download?.playable === true` (verified streaming available). Downloading is not a
removed-file badge. Display the actual transfer state and percentage only when
size is known. `playable` is false for paused, queued, checking, removed, errored,
disabled or not-yet-restored transfers. `identity_changed` means the transfer was
reidentified after this library entry was created. Playback must not resume it
or bypass queue, storage, selection or bandwidth limits.

Full import replaces the source behind the same item ID, preserving user progress,
favorites and reactions. The download field disappears after import. Existing
completed transfers with intentionally removed media do not recreate library
entries unless the owner explicitly requests a reimport.

## Player

Use the normal `POST /api/v1/playback/sessions` and existing progress endpoints.
Sessions include optional `progressive: true`; starting can wait for verified
pieces. The ready session carries the probed duration. Seeking creates a new HLS
session or sends an HTTP range for direct play, reprioritizing pieces near that
position. Peers and bandwidth still determine how quickly playback can buffer;
there is no promise that every format or transfer can start immediately.

`GET`/`HEAD /api/v1/items/{id}/stream` remains authenticated and library-authorized.
It accepts a single byte range (closed, open-ended or suffix), supports seeking,
and waits for verified pieces instead of reading preallocated holes. Multi-ranges
and unsatisfiable ranges return 416. There are at most eight active torrent
readers; excess requests return 429. Disconnecting drops the read priorities.
The engine reserves ten disk-work permits: eight possible reader leases plus two
for initialization and piece processing, so readers cannot exhaust that budget.
Paused/removed transfers stop readers; GET never starts/resumes a download.

FFprobe and FFmpeg conversion use a temporary, authenticated loopback range
reader, not a sparse file or a public torrent-engine API. It permits media
demuxers only, disables inherited HTTP proxies, and closes with its worker.
Growing torrent sources do not enter the completed-file transcode cache.

## Frontend behavior

- Library movie/show plates, detail pages and episode rows show a compact
  downloading/paused/waiting indicator; series episodes can be opened separately.
- Play and Resume use local availability **or** `download.playable`. Missing-media
  Find again and removal controls apply to retained local entries, not pending
  downloads. Direct downloading removal is handled from Downloads.
- Download rows can link to their library items (a catalog lookup by download ID),
  without constructing IDs or issuing any new mutation.
- Refresh catalog status while the relevant view is visible, with no overlapping
  requests and cleanup on unmount. Do not add global high-frequency polling.
- The player explains buffering from a download, preserving its existing cancel,
  retry, seek, progress and permission behavior. It never downloads on mount.
- Watch-together support is unchanged; do not advertise reliable synchronized
  progressive playback in this slice.

This is an additive contract. UI verification against live media is separate from
the generated-fixture and type/build checks.

The verified-reader and seek-priority behavior comes from the pinned
[librqbit 9.1.0 streaming implementation](https://github.com/ikatson/rqbit/blob/v9.1.0/crates/librqbit/src/torrent_state/streaming.rs).
Generated loopback-peer tests cover waiting for verification, seeking before full
completion, reader limits, cancellation and the private FFmpeg/HLS bridge.
