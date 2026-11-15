# Glossary

Use these terms consistently in code, API fields and documentation.

| Term | Meaning |
| --- | --- |
| Library | Catalog of media found under configured library roots. |
| Library root | Configured movie or TV folder scanned without modifying its files. |
| Item | Catalog entry such as a movie, show, season or episode; its ID is independent of a path. |
| Media file | File containing video, audio and/or subtitle streams. An item may have multiple files. |
| Track | One selectable video, audio or subtitle stream within a media file. |
| Scan | Discover and reconcile files under a library root. An unavailable root is not evidence of deletion. |
| Probe | Inspect a file's container, codecs, duration and tracks using ffprobe. |
| Container | Format holding streams, such as MP4 or Matroska (MKV). |
| Codec | Encoding format of a stream, such as H.264, HEVC or AAC. |
| Direct play | Deliver the original file without changing its container or encoded streams. |
| Remux | Change the container while copying encoded streams without re-encoding them. |
| Transcode | Decode and re-encode one or more streams for client compatibility. |
| HLS | HTTP Live Streaming; a playlist (manifest) references media segments. |
| Playback session | Authorized server-managed playback instance with selected tracks and delivery method. |
| Progress | Per-user saved playback position and completion state. |
| Scratch | Disposable working storage, such as generated transcode segments. |
| Hardware acceleration | Use a GPU for supported decoding/encoding steps. NVIDIA support must be verified on the host. |
| Request | Household member's request to obtain a movie or series. |
| Manager | Movie/series module that tracks wanted media and coordinates acquisition and import. |
| Indexer | Configured source/API searched for downloadable releases. |
| Media release | Download candidate returned by an indexer. Distinct from an Oxevu software release. |
| Download job | Tracked transfer, later handled by the BitTorrent module. |
| Import | Deliberate placement of completed media into a library root, using configured copy/hardlink rules. |
| Seeding | Uploading torrent data to peers after or during download; source files may need to remain. |
| Show slug | Readable URL segment such as `arcane`. Duplicate titles receive numeric suffixes; adding or renaming shows can change these links. Internal IDs and progress remain unchanged. |
| Watch party | Authorized group sharing a server-controlled timeline; each member has an independent playback session and library permissions. |
| Playback lease | Heartbeat-renewed session lifetime; idle sessions are stopped and cleaned up. |
| Software release | Dated Oxevu version recorded in VERSION and CHANGELOG.md, starting at 0.1.0. |
| vd | Home server and target for playback/resource validation. |

Service replacements: Jellyfin (library/player), Seerr/Jellyseerr (requests),
Radarr (movies), Sonarr (series), Prowlarr (indexers), qBittorrent (BitTorrent).

- **Provider:** an optional external metadata, artwork or subtitle service. Its
  application credentials belong to Oxevu, independently of Jellyfin.
- **WebVTT:** timed text captions browsers can display separately from video.
- **Quality preset:** a per-stream resolution and bitrate ceiling, bounded by
  the administrator's server limits.
